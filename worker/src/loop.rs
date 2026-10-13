//! Worker loop implementation for digital-product-studio
//!
//! Contains the core worker loop logic separated from main.rs for testability.

use pipeline_core::queue::{claim_job, mark_running, mark_done, mark_failed, DEFAULT_LEASE_SECONDS, MAX_RETRIES};
use pipeline_core::db::entities::{Job, JobTool, JobStatus};
use pipeline_core::error::{AppError, Result};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error, instrument};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WorkerLoopConfig {
    pub lease_seconds: i64,
    pub poll_base_ms: u64,
    pub poll_jitter_ms: u64,
}

impl Default for WorkerLoopConfig {
    fn default() -> Self {
        Self {
            lease_seconds: DEFAULT_LEASE_SECONDS,
            poll_base_ms: 500,
            poll_jitter_ms: 1500,
        }
    }
}

pub struct WorkerLoop {
    pool: Arc<PgPool>,
    config: WorkerLoopConfig,
    // Semaphores for tool concurrency control
    cf_image_gen_permit: Arc<tokio::sync::Semaphore>,
    photocraft_permit: Arc<tokio::sync::Semaphore>,
    vectorcraft_permit: Arc<tokio::sync::Semaphore>,
}

impl WorkerLoop {
    pub fn new(pool: Arc<PgPool>, config: WorkerLoopConfig) -> Self {
        Self {
            pool: pool.clone(),
            config: config.clone(),
            cf_image_gen_permit: Arc::new(tokio::sync::Semaphore::new(2)),
            photocraft_permit: Arc::new(tokio::sync::Semaphore::new(4)),
            vectorcraft_permit: Arc::new(tokio::sync::Semaphore::new(4)),
        }
    }

    /// Run a single iteration of the worker loop
    /// Returns Ok(true) if a job was processed, Ok(false) if no jobs available, Err on error
    #[instrument(skip(self))]
    pub async fn run_once(&self) -> Result<bool, AppError> {
        // 1. CLAIM - atomic transaction with SKIP LOCKED
        let job = claim_job(&self.pool, self.config.lease_seconds).await?;
        
        let job = match job {
            Some(job) => job,
            None => return Ok(false), // No jobs available
        };

        info!("Claimed job {} (tool: {:?}, attempt {})", job.id, job.tool, job.attempts);

        // 2. Get semaphore for tool class
        let _permit = self.acquire_tool_permit(&job.tool).await?;

        // 3. MARK RUNNING
        let job = mark_running(&self.pool, job.id).await?;
        info!("Job {} marked as running", job.id);

        // 4. EXECUTE - spawn container and run command
        let receipt = crate::exec::execute_job(&job, &self.pool).await;

        // 5. REPORT - mark done/failed based on result
        match receipt {
            Ok(_receipt) => {
                let job = mark_done(&self.pool, job.id).await?;
                info!("Job {} completed successfully", job.id);
            }
            Err(e) => {
                error!("Job {} failed: {:?}", job.id, e);
                let job = mark_failed(&self.pool, job.id, &e.to_string(), MAX_RETRIES).await?;
                info!("Job {} marked as failed (attempt {})", job.id, job.attempts);
            }
        }

        Ok(true)
    }

    /// Run the worker loop continuously
    #[instrument(skip(self))]
    pub async fn run_continuous(&self) -> Result<(), AppError> {
        info!("Worker loop started");
        
        loop {
            match self.run_once().await {
                Ok(true) => {
                    // Job processed, continue immediately
                }
                Ok(false) => {
                    // No jobs available, backoff with jitter
                    let delay = self.jittered_backoff();
                    tracing::trace!("No jobs available, sleeping {:?}", delay);
                    sleep(delay).await;
                }
                Err(e) => {
                    error!("Worker loop error: {:?}", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Acquire semaphore permit for tool class
    async fn acquire_tool_permit(&self, tool: &JobTool) -> Result<tokio::sync::OwnedSemaphorePermit, AppError> {
        match tool {
            JobTool::CfImageGen => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {e}")))?),
            JobTool::PhotocraftCli => Ok(self.photocraft_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {e}")))?),
            JobTool::VectorcraftCli => Ok(self.vectorcraft_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {e}")))?),
            JobTool::WebhookProcessor => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {e}")))?),
            JobTool::ProductAssembly => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {e}")))?),
            JobTool::PublishListing => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {e}")))?),
        }
    }

    /// Jittered backoff when queue is empty
    fn jittered_backoff(&self) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=self.config.poll_jitter_ms);
        Duration::from_millis(self.config.poll_base_ms + jitter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test(migrations = "../pipeline-core/migrations")]
    async fn test_worker_loop_single_iteration(pool: PgPool) {
        // Test that run_once processes a job when available
        let pool = Arc::new(pool);
        let loop_ = WorkerLoop::new(pool.clone(), WorkerLoopConfig::default());
        
        // Setup test data
        let niche_id = sqlx::query_scalar!(
            "INSERT INTO niches (name, vertical, demand_signals, competition_score)
             VALUES ('test', 'printable_wall_art', '{}', 0)
             RETURNING id"
        )
        .fetch_one(&*pool)
        .await
        .unwrap();

        let run_id = sqlx::query_scalar!(
            "INSERT INTO production_runs (niche_id, status, target_count, params)
             VALUES ($1, 'generating', 1, '{}')
             RETURNING id",
            niche_id
        )
        .fetch_one(&*pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO jobs (run_id, tool, command_spec, status)
             VALUES ($1, 'cf-image-gen', '{}', 'queued')",
            run_id
        )
        .execute(&*pool)
        .await
        .unwrap();

        // Run once should process the job
        let processed = loop_.run_once().await.unwrap();
        assert!(processed, "Should have processed a job");
    }

    #[ignore]
    #[sqlx::test(migrations = "../pipeline-core/migrations")]
    async fn test_worker_loop_empty_queue(pool: PgPool) {
        // Test that run_once returns false when queue is empty
        let pool = Arc::new(pool);
        let loop_ = WorkerLoop::new(pool.clone(), WorkerLoopConfig::default());
        
        // No jobs in queue
        let processed = loop_.run_once().await.unwrap();
        assert!(!processed, "Should return false when no jobs available");
    }
}