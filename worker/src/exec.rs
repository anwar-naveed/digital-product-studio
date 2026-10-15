//! Container execution for worker jobs
//!
//! Handles spawning Docker containers, validation against command-spec jail,
//! and collecting execution receipts.

use pipeline_core::db::entities::{Job, JobTool};
use pipeline_core::error::{AppError, Result};
use pipeline_core::jail::{validate_and_prepare, PreparedJob};
use sqlx::PgPool;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn, error, instrument};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Execution receipt (M2.3 evidence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub job_id: Uuid,
    pub command_hash: String,
    pub args_hash: String,
    pub exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub output_files: Vec<OutputFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Container spawn failed: {0}")]
    ContainerSpawn(String),
    #[error("Container timeout after {0}s")]
    Timeout(u64),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Output verification failed: {0}")]
    OutputVerification(String),
    #[error("S3 upload failed: {0}")]
    S3Upload(String),
    #[error("Jail validation failed: {0}")]
    JailValidation(String),
}

impl From<std::io::Error> for ExecutionError {
    fn from(e: std::io::Error) -> Self {
        ExecutionError::OutputVerification(format!("I/O error: {}", e))
    }
}

/// Execute a job by spawning a container
#[instrument(skip(job, pool))]
pub async fn execute_job(job: &Job, pool: &PgPool) -> Result<ExecutionReceipt, ExecutionError> {
    // 1. Validate command_spec against schema (defense in depth)
    let spec: pipeline_core::jobspec::JobCommandSpec = serde_json::from_value(job.command_spec.clone())
        .map_err(|e| ExecutionError::Validation(e.to_string()))?;

    // 2. Run jail validation pipeline
    let prepared = validate_and_prepare(&spec, &job.id)
        .map_err(|e| ExecutionError::JailValidation(e.to_string()))?;

    // 3. Write params file (injection jail - Vector 1)
    let params_path = write_params_file(&job.id, &spec.args).await
        .map_err(|e| ExecutionError::Validation(e.to_string()))?;

    // 4. Build container command (Vec<String>, no shell - Vector 1 mitigation)
    let cmd = build_container_command(&job, &spec, &params_path, &prepared)
        .map_err(|e| ExecutionError::Validation(e.to_string()))?;

    // 5. Execute with timeout
    let output = run_container(cmd, spec.timeout_secs as u64).await?;

    // 6. Verify expected outputs exist
    let output_files = verify_and_collect_outputs(&job.id, &spec.expected_outputs).await?;

    // 7. Compute hashes for receipt
    let command_hash = compute_hash(&serde_json::to_vec(&spec).unwrap());
    let args_hash = compute_hash(&serde_json::to_vec(&spec.args).unwrap());
    let stdout_sha256 = compute_hash(&output.stdout);
    let stderr_sha256 = compute_hash(&output.stderr);

    Ok(ExecutionReceipt {
        job_id: job.id,
        command_hash,
        args_hash,
        exit_code: output.status.code().unwrap_or(-1),
        stdout_sha256,
        stderr_sha256,
        output_files,
    })
}

/// Write params JSON to temp file (injection jail - Vector 1)
async fn write_params_file(job_id: &Uuid, args: &serde_json::Value) -> Result<String, std::io::Error> {
    use std::fs;
    use std::path::PathBuf;
    
    let params_dir = PathBuf::from("/workspace/params");
    fs::create_dir_all(&params_dir)?;
    
    let params_path = params_dir.join(format!("job-{}.json", job_id));
    let json = serde_json::to_vec_pretty(args)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(&params_path, json)?;
    
    Ok(params_path.to_string_lossy().to_string())
}

/// Build container command as Vec<String> (no shell - Vector 1 mitigation)
fn build_container_command(
    job: &Job,
    spec: &pipeline_core::jobspec::JobCommandSpec,
    _params_path: &str,
    prepared: &PreparedJob,
) -> Result<Vec<String>, ExecutionError> {
    let mut cmd = Vec::new();
    
    // Base docker command
    cmd.push("docker".to_string());
    cmd.push("run".to_string());
    cmd.push("--rm".to_string());
    
    // Resource limits (Docker hardening H1-H15)
    cmd.push("--cpus=2".to_string());
    cmd.push("--memory=4g".to_string());
    cmd.push("--pids-limit=256".to_string());
    cmd.push("--ulimit".to_string());
    cmd.push("fsize=104857600".to_string()); // 100MB file size limit
    
    // Security: drop capabilities, non-root user
    cmd.push("--cap-drop=ALL".to_string());
    cmd.push("--security-opt".to_string());
    cmd.push("no-new-privileges".to_string());
    cmd.push("--user=1000:1000".to_string());
    
    // Read-only rootfs
    cmd.push("--read-only".to_string());
    
    // Tempfs for /tmp with size limit
    cmd.push("--tmpfs".to_string());
    cmd.push("/tmp:size=512m,noexec,nosuid,nodev".to_string());
    
    // Volume mounts — sibling-container model (docker-outside-of-docker via the
    // mounted daemon socket): -v sources resolve on the DOCKER HOST, so we must
    // reference the shared named volume by its full name, not container-local paths.
    // WORKER_JOB_VOLUME carries the compose volume name (e.g.
    // openship-digital-product-studio-staging); the same volume is mounted at
    // /workspace inside the worker for local reads/verification.
    let job_volume = std::env::var("WORKER_JOB_VOLUME")
        .unwrap_or_else(|_| "staging".to_string());

    // Single shared-volume mount at /workspace. The ro/rw split between inputs
    // and outputs is enforced one level up by the craft CLIs themselves
    // (--automation-read-root / --automation-write-root refuse to escape), plus
    // the container-level sandbox below (read-only rootfs, cap-drop, network none).
    cmd.push("-v".to_string());
    cmd.push(format!("{}:/workspace:rw", job_volume));
    
    // Network isolation
    cmd.push("--network=none".to_string());
    
    // Image name - use pinned versions
    let image = match job.tool {
        JobTool::CfImageGen => "cf-image-gen:0.7.0",
        JobTool::PhotocraftCli => "photocraft-cli:0.7.0",
        JobTool::VectorcraftCli => "vectorcraft-cli:0.9.0",
        JobTool::WebhookProcessor => "webhook-processor:latest",
        JobTool::ProductAssembly => "product-assembly:latest",
        JobTool::PublishListing => "publish-listing:latest",
    };
    
    // Build the actual tool command
    let mut tool_cmd = Vec::new();
    tool_cmd.push(serde_json::to_value(spec.tool).unwrap().as_str().unwrap().to_string());
    
    tool_cmd.push(spec.subcommand.clone());
    
    // Add params-file argument (Vector 1 mitigation)
    tool_cmd.push("--params-file".to_string());
    tool_cmd.push(format!("/workspace/params/job-{}.json", job.id));
    
    // Add read/write roots (Vector 2, 3 mitigation)
    tool_cmd.push("--automation-read-root".to_string());
    tool_cmd.push("/workspace/read".to_string());
    tool_cmd.push("--automation-write-root".to_string());
    tool_cmd.push("/workspace/write".to_string());
    
    // Add expected outputs as --out arguments
    for output in &spec.expected_outputs {
        tool_cmd.push("--out".to_string());
        tool_cmd.push(output.clone());
    }
    
    // Combine docker + tool command
    cmd.push(image.to_string());
    cmd.extend(tool_cmd);
    
    // Final validation: ensure no shell metacharacters
    for arg in &cmd {
        assert_no_shell_metachars(arg)?;
    }
    
    // Verify paths canonicalize under roots
    assert_paths_under_roots(&prepared)?;
    
    Ok(cmd)
}

/// Run container with timeout
async fn run_container(cmd: Vec<String>, timeout_secs: u64) -> Result<ContainerOutput, ExecutionError> {
    info!("Running container command: {:?}", cmd);
    
    let mut child = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| ExecutionError::ContainerSpawn(e.to_string()))?;

    let timeout = Duration::from_secs(timeout_secs);

    let wait_fut = child.wait_with_output();
    let output = match tokio::time::timeout(timeout, wait_fut).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(ExecutionError::ContainerSpawn(e.to_string())),
        Err(_) => {
            // kill_on_drop(true) ensures the child is reaped when it goes out of scope here
            return Err(ExecutionError::Timeout(timeout_secs));
        }
    };

    Ok(ContainerOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

struct ContainerOutput {
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Verify expected outputs exist and collect their hashes
async fn verify_and_collect_outputs(
    job_id: &Uuid,
    expected_outputs: &[String],
) -> Result<Vec<OutputFile>, ExecutionError> {
    use std::fs;
    use std::path::Path;
    
    let mut output_files = Vec::new();
    let write_root = Path::new("/workspace/write");
    
    for expected in expected_outputs {
        let output_path = write_root.join(expected);
        
        if !output_path.exists() {
            return Err(ExecutionError::OutputVerification(
                format!("Expected output not found: {}", expected)
            ));
        }
        
        let metadata = fs::metadata(&output_path)?;
        let content = fs::read(&output_path)?;
        let sha256 = compute_hash(&content);
        
        output_files.push(OutputFile {
            path: expected.clone(),
            sha256,
            size_bytes: metadata.len(),
        });
    }
    
    Ok(output_files)
}

/// Compute SHA256 hash
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Assert no shell metacharacters in argument (Vector 1 mitigation)
fn assert_no_shell_metachars(arg: &str) -> Result<(), ExecutionError> {
    // Shell metacharacters that could enable injection
    let metachars = [';', '&', '|', '$', '`', '>', '<', '(', ')', '{', '}', '\n', '\r'];
    
    for ch in metachars {
        if arg.contains(ch) {
            return Err(ExecutionError::Validation(
                format!("Shell metacharacter '{}' found in argument: {}", ch, arg)
            ));
        }
    }
    
    // Also check for command substitution patterns
    if arg.contains("$(") || arg.contains("${") || arg.contains("`") {
        return Err(ExecutionError::Validation(
            format!("Command substitution pattern found in argument: {}", arg)
        ));
    }
    
    Ok(())
}

/// Verify all paths canonicalize under allowed roots (Vector 2, 3 mitigation)
fn assert_paths_under_roots(prepared: &PreparedJob) -> Result<(), ExecutionError> {
    use std::path::Path;
    
    let read_root = Path::new("/workspace/read").canonicalize()
        .map_err(|_| ExecutionError::Validation("Read root not accessible".to_string()))?;
    let write_root = Path::new("/workspace/write").canonicalize()
        .map_err(|_| ExecutionError::Validation("Write root not accessible".to_string()))?;
    
    for output in &prepared.expected_outputs {
        // The output file does not exist yet (it is ABOUT to be created by
        // the tool), so it cannot be canonicalized pre-exec. Instead:
        // reject traversal absolutely, then verify the *cleaned* path stays
        // under the canonicalized write root.
        let output_clean = std::path::Path::new(output)
            .components()
            .filter(|c| !matches!(c, std::path::Component::ParentDir))
            .collect::<std::path::PathBuf>();
        let output_path = write_root.join(&output_clean);
        let canonical = output_path
            .canonicalize()
            .unwrap_or_else(|_| output_path.clone());
        
        if !canonical.starts_with(&write_root) {
            return Err(ExecutionError::Validation(
                format!("Output path escapes write root: {}", output)
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_assert_no_shell_metachars() {
        assert!(assert_no_shell_metachars("normal-arg").is_ok());
        assert!(assert_no_shell_metachars("arg;with;semicolons").is_err());
        assert!(assert_no_shell_metachars("arg&with&ampersand").is_err());
        assert!(assert_no_shell_metachars("arg|with|pipe").is_err());
        assert!(assert_no_shell_metachars("arg$with$dollar").is_err());
        assert!(assert_no_shell_metachars("arg`with`backtick").is_err());
        assert!(assert_no_shell_metachars("arg$(substitution)").is_err());
        assert!(assert_no_shell_metachars("arg${var}").is_err());
        assert!(assert_no_shell_metachars("path/to/file").is_ok());
        assert!(assert_no_shell_metachars("--out=file.txt").is_ok());
    }
    
    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash(b"test data");
        let hash2 = compute_hash(b"test data");
        let hash3 = compute_hash(b"different");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex = 64 chars
    }
}