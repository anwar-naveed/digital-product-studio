//! Post-job pipeline advancement: upload outputs to object storage and
//! advance run/design state so curation + the UI reflect real progress.

use pipeline_core::db::entities::Job;
use pipeline_core::s3::S3Client;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostProcessConfig {
    pub staging_root: PathBuf,
    pub s3_client: Option<S3Client>,
}

impl std::fmt::Debug for PostProcessConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostProcessConfig")
            .field("staging_root", &self.staging_root)
            .field("s3_client", &self.s3_client.is_some())
            .finish()
    }
}

/// Before a sandboxed job runs: stage any S3 inputs it declares in its
/// command-spec args (product_assembly jobs carry `curated_key`). Tools run
/// with network "none", so their inputs must be placed on the shared volume
/// read side first. Idempotent — skips keys already staged.
pub async fn stage_job_inputs(job: &Job, cfg: &PostProcessConfig) -> Result<(), anyhow::Error> {
    let Some(s3) = cfg.s3_client.as_ref() else {
        return Ok(()); // s3-integration disabled: nothing to stage
    };
    let curated_key = match job
        .command_spec
        .get("args")
        .and_then(|a| a.get("curated_key"))
        .and_then(|v| v.as_str())
    {
        Some(k) if !k.is_empty() => k.to_string(),
        _ => return Ok(()), // not an input-carrying job (e.g. cf-image-gen)
    };

    let dest = cfg
        .staging_root
        .join("read")
        .join("inputs")
        .join(&curated_key);
    if tokio::fs::try_exists(&dest).await.unwrap_or(false) {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let result = s3
        .get(&curated_key)
        .await
        .map_err(|e| anyhow::anyhow!("s3 get {}: {}", curated_key, e))?;
    let data = result
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("s3 read {}: {}", curated_key, e))?;
    tokio::fs::write(&dest, &data).await?;
    tracing::info!(key = %curated_key, "staged assembly input for job {}", job.id);
    Ok(())
}

/// After a cf-image-gen job completes: read the produced files from the
/// staging write root, upload them to S3 and record design_variants rows.
pub async fn ingest_design_outputs(
    pool: &PgPool,
    cfg: &PostProcessConfig,
    job_id: Uuid,
) -> Result<(), anyhow::Error> {
    let Some(s3) = cfg.s3_client.as_ref() else {
        return Ok(()); // s3-integration disabled: skip upload silently
    };

    // job -> args (design_id, output)
    let row: Option<serde_json::Value> = sqlx::query_scalar!(
        r#"SELECT command_spec->'args' as "args!: serde_json::Value" FROM jobs WHERE id = $1"#,
        job_id
    )
    .fetch_optional(pool)
    .await?;
    let Some(args) = row else { return Ok(()) };
    let Some(design_id) = args.get("design_id").and_then(|v| v.as_str()) else {
        return Ok(());
    };
    let design_id: Uuid = design_id
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid design uuid: {}", e))?;
    let outputs: Vec<String> = args
        .get("output")
        .and_then(|v| v.as_str())
        .map(|s| vec![s.to_string()])
        .unwrap_or_default();

    // Assembly jobs (product_assembly) declare their outputs at the spec
    // level instead of args.output. They upload bundles but are NOT design
    // variants, so the design_variants insert is skipped for them.
    let is_assembly = args.get("curated_key").map(|_| true).unwrap_or(false);
    let outputs = if outputs.is_empty() {
        let eo: Option<serde_json::Value> = sqlx::query_scalar!(
            r#"SELECT command_spec->'expected_outputs' as "eo!: serde_json::Value" FROM jobs WHERE id = $1"#,
            job_id
        )
        .fetch_optional(pool)
        .await?;
        eo.and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .unwrap_or_default()
    } else {
        outputs
    };

    for rel in outputs {
        if rel.is_empty() { continue; }
        let path = cfg.staging_root.join("write").join(&rel);
        let Ok(data) = tokio::fs::read(&path).await else {
            tracing::warn!("output missing for ingest: {}", path.display());
            continue;
        };
        let sha = sha256_hex(&data);
        // rel already carries the design prefix (<design_id>/<variant>.png) — use it as-is.
        let s3_key = rel.trim_start_matches('/').to_string();
        let mut meta = HashMap::new();
        meta.insert("sha256".to_string(), sha.clone());
        s3.put(&s3_key, data, meta).await?;

        if is_assembly {
            // bundles are products, not variants — skip the variants table.
            tracing::info!(key = %s3_key, "uploaded assembly bundle");
            continue;
        }

        sqlx::query!(
            "INSERT INTO design_variants (design_id, kind, s3_key, sha256)
             VALUES ($1, 'raw', $2, $3)
             ON CONFLICT (sha256) DO NOTHING",
            design_id,
            s3_key,
            sha,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Advance production_runs status once its jobs reach terminal states.
/// all done -> 'curating'; any dead -> 'failed'; else 'generating'.
pub async fn advance_run_status(pool: &PgPool, run_id: Uuid) -> Result<(), anyhow::Error> {
    let counts = sqlx::query!(
        r#"
        SELECT
            count(*) FILTER (WHERE status = 'done') AS done,
            count(*) FILTER (WHERE status = 'dead') AS dead,
            count(*) FILTER (WHERE status IN ('queued','running')) AS open
        FROM jobs WHERE run_id = $1
        "#,
        run_id
    )
    .fetch_one(pool)
    .await?;

    let new_status = if counts.open.unwrap_or(0) > 0 {
        "generating"
    } else if counts.dead.unwrap_or(0) > 0 {
        "failed"
    } else {
        "curating"
    };

    sqlx::query!(
        "UPDATE production_runs
         SET status = $2,
             started_at = COALESCE(started_at, now())
         WHERE id = $1",
        run_id,
        new_status
    )
    .execute(pool)
    .await?;
    Ok(())
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    hex(&h.finalize())
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}
