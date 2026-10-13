//! Job queue implementation for digital-product-studio
//!
//! Provides atomic claim with SKIP LOCKED, reaper for stale jobs,
//! and exponential backoff with cap at 3 retries.

use crate::db::entities::{Job, JobStatus, JobTool};
use crate::error::{AppError, Result};
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

/// Default lease duration in seconds (120s per spec)
pub const DEFAULT_LEASE_SECONDS: i64 = 120;

/// Maximum retry attempts (cap at 3 per spec)
pub const MAX_RETRIES: i32 = 3;

/// Base backoff interval in seconds (30s per spec)
pub const BASE_BACKOFF_SECONDS: i64 = 30;

/// Claim a job atomically using SELECT ... FOR UPDATE SKIP LOCKED
///
/// This implements the G1 requirement: single transaction, no double-claim.
/// The claim is atomic - only one worker can claim a given job.
pub async fn claim_job(
    pool: &PgPool,
    lease_seconds: i64,
) -> Result<Option<Job>, AppError> {
    let now = Utc::now();
    let lock_expires_at = now + Duration::seconds(lease_seconds);

    let job = sqlx::query_as!(
        Job,
        r#"
        UPDATE jobs
        SET status = 'claimed',
            started_at = $1,
            lock_expires_at = $2,
            attempts = attempts + 1
        WHERE id = (
            SELECT id FROM jobs
            WHERE status = 'queued'
              AND (next_attempt_at IS NULL OR next_attempt_at <= $1)
            ORDER BY queued_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
        "#,
        now,
        lock_expires_at
    )
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

/// Mark a job as running (transition from claimed to running)
pub async fn mark_running(pool: &PgPool, job_id: Uuid) -> Result<Job, AppError> {
    let job = sqlx::query_as!(
        Job,
        r#"
        UPDATE jobs
        SET status = 'running'
        WHERE id = $1
        RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
        "#,
        job_id
    )
    .fetch_one(pool)
    .await?;

    Ok(job)
}

/// Mark a job as done (successful completion)
pub async fn mark_done(pool: &PgPool, job_id: Uuid) -> Result<Job, AppError> {
    let job = sqlx::query_as!(
        Job,
        r#"
        UPDATE jobs
        SET status = 'done',
            finished_at = $2,
            lock_expires_at = NULL
        WHERE id = $1
        RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
        "#,
        job_id,
        Utc::now()
    )
    .fetch_one(pool)
    .await?;

    Ok(job)
}

/// Mark a job as failed with error, applying exponential backoff
///
/// Implements G3: backoff matches 30s*2^attempts cap 3
/// - attempt 0: 30s * 2^0 = 30s
/// - attempt 1: 30s * 2^1 = 60s
/// - attempt 2: 30s * 2^2 = 120s
/// - attempt 3: cap at max retries -> dead
pub async fn mark_failed(
    pool: &PgPool,
    job_id: Uuid,
    error: &str,
    max_retries: i32,
) -> Result<Job, AppError> {
    let now = Utc::now();

    // Calculate backoff: 30s * 2^attempts
    // We need to get the current attempts first
    let job = sqlx::query!(
        "SELECT attempts FROM jobs WHERE id = $1",
        job_id
    )
    .fetch_one(pool)
    .await?;

    let attempts = job.attempts;
    let next_attempt_at = if attempts < max_retries {
        let backoff_seconds = BASE_BACKOFF_SECONDS * (2_i64.pow(attempts as u32));
        Some(now + Duration::seconds(backoff_seconds))
    } else {
        None
    };

    let status = if attempts >= max_retries {
        JobStatus::Dead
    } else {
        JobStatus::Failed
    };

    let finished_at = if attempts >= max_retries {
        Some(now)
    } else {
        None
    };

    let job = sqlx::query_as!(
        Job,
        r#"
        UPDATE jobs
        SET status = $2,
            last_error = $3,
            finished_at = $4,
            next_attempt_at = $5,
            lock_expires_at = NULL
        WHERE id = $1
        RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
        "#,
        job_id,
        status as JobStatus,
        error,
        finished_at,
        next_attempt_at
    )
    .fetch_one(pool)
    .await?;

    Ok(job)
}

/// Reaper query: expire stale running/claimed jobs
///
/// Implements G2: reaper expires stale running jobs
/// Jobs with lock_expires_at in the past are moved back to failed/queued
/// with incremented attempts and backoff applied.
pub async fn reap_stale_jobs(
    pool: &PgPool,
    max_retries: i32,
) -> Result<Vec<Job>, AppError> {
    let now = Utc::now();

    // First, find jobs with expired leases
    let stale_jobs = sqlx::query!(
        r#"
        SELECT id, attempts FROM jobs
        WHERE status IN ('claimed', 'running')
          AND lock_expires_at IS NOT NULL
          AND lock_expires_at < $1
        "#,
        now
    )
    .fetch_all(pool)
    .await?;

    let mut reaped = Vec::new();

    for record in stale_jobs {
        let job_id = record.id;
        let attempts = record.attempts;

        // Calculate backoff for re-queue
        let next_attempt_at = if attempts < max_retries {
            let backoff_seconds = BASE_BACKOFF_SECONDS * (2_i64.pow(attempts as u32));
            Some(now + Duration::seconds(backoff_seconds))
        } else {
            None
        };

        let status = if attempts >= max_retries {
            JobStatus::Dead
        } else {
            JobStatus::Failed
        };

        let finished_at = if attempts >= max_retries {
            Some(now)
        } else {
            None
        };

        let error_msg = "Job lease expired - reaper moved to failed";

        let job = sqlx::query_as!(
            Job,
            r#"
            UPDATE jobs
            SET status = $2,
                last_error = $3,
                finished_at = $4,
                next_attempt_at = $5,
                lock_expires_at = NULL
            WHERE id = $1
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            job_id,
            status as JobStatus,
            error_msg,
            finished_at,
            next_attempt_at
        )
        .fetch_one(pool)
        .await?;

        reaped.push(job);
    }

    Ok(reaped)
}

/// Queue statistics for monitoring
#[derive(Debug, Clone, FromRow)]
pub struct QueueStats {
    pub status: JobStatus,
    pub count: Option<i64>,
    pub oldest: Option<DateTime<Utc>>,
    pub newest: Option<DateTime<Utc>>,
}

pub async fn get_queue_stats(pool: &PgPool) -> Result<Vec<QueueStats>, AppError> {
    let stats = sqlx::query_as!(
        QueueStats,
        r#"
        SELECT 
            status as "status: JobStatus",
            count(*) as count,
            min(queued_at) as oldest,
            max(queued_at) as newest
        FROM jobs
        GROUP BY status
        ORDER BY status
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

/// Tool concurrency statistics
#[derive(Debug, Clone, FromRow)]
pub struct ToolConcurrencyStats {
    pub tool: JobTool,
    pub status: JobStatus,
    pub count: Option<i64>,
}

pub async fn get_tool_concurrency_stats(pool: &PgPool) -> Result<Vec<ToolConcurrencyStats>, AppError> {
    let stats = sqlx::query_as!(
        ToolConcurrencyStats,
        r#"
        SELECT 
            tool as "tool: JobTool",
            status as "status: JobStatus",
            count(*) as count
        FROM jobs
        WHERE status IN ('claimed', 'running')
        GROUP BY tool, status
        ORDER BY tool, status
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // These tests require a running PostgreSQL instance
    // They should be run inside a Docker container with postgres:16
    
    #[ignore]
    #[sqlx::test]
    async fn test_claim_job_atomic(pool: PgPool) {
        // This test verifies G1: Claim = single txn, no double-claim
        // Run with two concurrent claimers inside Docker pg
        
        // Setup: create a niche and production run
        let niche_id = sqlx::query_scalar!(
            "INSERT INTO niches (name, vertical, demand_signals, competition_score)
             VALUES ('test', 'printable_wall_art', '{}', 0)
             RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let run_id = sqlx::query_scalar!(
            "INSERT INTO production_runs (niche_id, status, target_count, params)
             VALUES ($1, 'generating', 1, '{}')
             RETURNING id",
            niche_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Create a job
        let job_id = sqlx::query_scalar!(
            "INSERT INTO jobs (run_id, tool, command_spec, status)
             VALUES ($1, 'cf-image-gen', '{}', 'queued')
             RETURNING id",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Two concurrent claims - only one should succeed
        let claim1 = claim_job(&pool, DEFAULT_LEASE_SECONDS).await.unwrap();
        let claim2 = claim_job(&pool, DEFAULT_LEASE_SECONDS).await.unwrap();

        // Exactly one should get the job, the other should get None
        let claimed_count = [claim1.is_some(), claim2.is_some()].iter().filter(|&&x| x).count();
        assert_eq!(claimed_count, 1, "Only one worker should claim the job");

        // The claimed job should have status 'claimed' and attempts = 1
        let claimed_job = claim1.or(claim2).unwrap();
        assert_eq!(claimed_job.status, JobStatus::Claimed);
        assert_eq!(claimed_job.attempts, 1);
    }

    #[ignore]
    #[sqlx::test]
    async fn test_reaper_expires_stale_jobs(pool: PgPool) {
        // This test verifies G2: Reaper expires stale running jobs
        
        let niche_id = sqlx::query_scalar!(
            "INSERT INTO niches (name, vertical, demand_signals, competition_score)
             VALUES ('test', 'printable_wall_art', '{}', 0)
             RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let run_id = sqlx::query_scalar!(
            "INSERT INTO production_runs (niche_id, status, target_count, params)
             VALUES ($1, 'generating', 1, '{}')
             RETURNING id",
            niche_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Create a job with expired lease
        let job_id = sqlx::query_scalar!(
            r#"
            INSERT INTO jobs (run_id, tool, command_spec, status, lock_expires_at, attempts)
            VALUES ($1, 'cf-image-gen', '{}', 'running', now() - interval '10 seconds', 0)
            RETURNING id
            "#,
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Run reaper
        let reaped = reap_stale_jobs(&pool, MAX_RETRIES).await.unwrap();

        assert_eq!(reaped.len(), 1);
        assert_eq!(reaped[0].id, job_id);
        assert_eq!(reaped[0].status, JobStatus::Failed);
        assert!(reaped[0].next_attempt_at.is_some());
        assert!(reaped[0].lock_expires_at.is_none());
    }

    #[ignore]
    #[sqlx::test]
    async fn test_backoff_matches_spec(pool: PgPool) {
        // This test verifies G3: Backoff matches 30s*2^attempts cap 3
        
        let niche_id = sqlx::query_scalar!(
            "INSERT INTO niches (name, vertical, demand_signals, competition_score)
             VALUES ('test', 'printable_wall_art', '{}', 0)
             RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let run_id = sqlx::query_scalar!(
            "INSERT INTO production_runs (niche_id, status, target_count, params)
             VALUES ($1, 'generating', 1, '{}')
             RETURNING id",
            niche_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Test attempt 0 -> backoff 30s
        let job_id = sqlx::query_scalar!(
            "INSERT INTO jobs (run_id, tool, command_spec, status, attempts)
             VALUES ($1, 'cf-image-gen', '{}', 'claimed', 0)
             RETURNING id",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let job = mark_failed(&pool, job_id, "test error", MAX_RETRIES).await.unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert!(job.next_attempt_at.is_some());
        let expected = Utc::now() + Duration::seconds(30);
        // Allow small delta for test execution time
        let diff = (job.next_attempt_at.unwrap() - expected).num_seconds().abs();
        assert!(diff < 5, "Backoff should be ~30s for attempt 0");

        // Test attempt 1 -> backoff 60s
        let job_id2 = sqlx::query_scalar!(
            "INSERT INTO jobs (run_id, tool, command_spec, status, attempts)
             VALUES ($1, 'cf-image-gen', '{}', 'claimed', 1)
             RETURNING id",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let job = mark_failed(&pool, job_id2, "test error", MAX_RETRIES).await.unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        let expected = Utc::now() + Duration::seconds(60);
        let diff = (job.next_attempt_at.unwrap() - expected).num_seconds().abs();
        assert!(diff < 5, "Backoff should be ~60s for attempt 1");

        // Test attempt 2 -> backoff 120s
        let job_id3 = sqlx::query_scalar!(
            "INSERT INTO jobs (run_id, tool, command_spec, status, attempts)
             VALUES ($1, 'cf-image-gen', '{}', 'claimed', 2)
             RETURNING id",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let job = mark_failed(&pool, job_id3, "test error", MAX_RETRIES).await.unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        let expected = Utc::now() + Duration::seconds(120);
        let diff = (job.next_attempt_at.unwrap() - expected).num_seconds().abs();
        assert!(diff < 5, "Backoff should be ~120s for attempt 2");

        // Test attempt 3 -> dead (max retries = 3)
        let job_id4 = sqlx::query_scalar!(
            "INSERT INTO jobs (run_id, tool, command_spec, status, attempts)
             VALUES ($1, 'cf-image-gen', '{}', 'claimed', 3)
             RETURNING id",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let job = mark_failed(&pool, job_id4, "test error", MAX_RETRIES).await.unwrap();
        assert_eq!(job.status, JobStatus::Dead);
        assert!(job.next_attempt_at.is_none());
        assert!(job.finished_at.is_some());
    }

    #[ignore]
    #[sqlx::test]
    async fn test_integration_enqueue_claim_complete(pool: PgPool) {
        // This test verifies G4: Integration test enqueue -> claim -> complete
        // Transitions: queued -> claimed -> running -> done
        
        let niche_id = sqlx::query_scalar!(
            "INSERT INTO niches (name, vertical, demand_signals, competition_score)
             VALUES ('test', 'printable_wall_art', '{}', 0)
             RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let run_id = sqlx::query_scalar!(
            "INSERT INTO production_runs (niche_id, status, target_count, params)
             VALUES ($1, 'generating', 1, '{}')
             RETURNING id",
            niche_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // ENQUEUE: Create a job
        let job = sqlx::query_as!(
            Job,
            r#"
            INSERT INTO jobs (run_id, tool, command_spec, status)
            VALUES ($1, 'cf-image-gen', '{}', 'queued')
            RETURNING id, run_id, tool as "tool: JobTool", command_spec, status as "status: JobStatus", attempts, queued_at, started_at, finished_at, last_error, next_attempt_at, lock_expires_at
            "#,
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(job.status, JobStatus::Queued);

        // CLAIM: Worker claims the job
        let claimed = claim_job(&pool, DEFAULT_LEASE_SECONDS).await.unwrap().unwrap();
        assert_eq!(claimed.id, job.id);
        assert_eq!(claimed.status, JobStatus::Claimed);
        assert!(claimed.lock_expires_at.is_some());

        // MARK RUNNING: Worker starts executing
        let running = mark_running(&pool, claimed.id).await.unwrap();
        assert_eq!(running.status, JobStatus::Running);

        // COMPLETE: Worker finishes successfully
        let done = mark_done(&pool, running.id).await.unwrap();
        assert_eq!(done.status, JobStatus::Done);
        assert!(done.finished_at.is_some());
        assert!(done.lock_expires_at.is_none());
    }
}