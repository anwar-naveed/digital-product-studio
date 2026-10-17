//! Worker binary entry point for digital-product-studio
//!
//! Runs the job queue worker loop with reaper background task.

use pipeline_core::queue::{claim_job, reap_stale_jobs, DEFAULT_LEASE_SECONDS, MAX_RETRIES};
use pipeline_core::db::entities::{Job, JobTool, JobStatus};
use pipeline_core::error::{AppError, Result};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, interval};
use tracing::{info, warn, error, trace, instrument};
use uuid::Uuid;

mod exec;
mod postprocess;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub lease_seconds: i64,
    pub poll_base_ms: u64,
    pub poll_jitter_ms: u64,
    pub max_cf_image_gen: usize,
    pub max_photocraft: usize,
    pub max_vectorcraft: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            lease_seconds: DEFAULT_LEASE_SECONDS,
            poll_base_ms: 500,
            poll_jitter_ms: 1500,
            max_cf_image_gen: 2,
            max_photocraft: 4,
            max_vectorcraft: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Worker {
    pool: Arc<PgPool>,
    config: WorkerConfig,
    post: Option<postprocess::PostProcessConfig>,
    // Semaphores for tool concurrency control
    cf_image_gen_permit: Arc<tokio::sync::Semaphore>,
    photocraft_permit: Arc<tokio::sync::Semaphore>,
    vectorcraft_permit: Arc<tokio::sync::Semaphore>,
}

impl Worker {
    pub async fn new(pool: Arc<PgPool>, config: WorkerConfig) -> Self {
        let post = Some(postprocess::PostProcessConfig {
            staging_root: std::path::PathBuf::from(
                std::env::var("STAGING_ROOT").unwrap_or_else(|_| "/workspace".into()),
            ),
            s3_client: build_s3_client().await,
        });
        Self {
            pool: pool.clone(),
            config: config.clone(),
            post,
            cf_image_gen_permit: Arc::new(tokio::sync::Semaphore::new(config.max_cf_image_gen)),
            photocraft_permit: Arc::new(tokio::sync::Semaphore::new(config.max_photocraft)),
            vectorcraft_permit: Arc::new(tokio::sync::Semaphore::new(config.max_vectorcraft)),
        }
    }

    /// Main entry point - starts worker loop and reaper
    #[instrument(skip(self))]
    pub async fn run(self) -> Result<(), AppError> {
        info!("Worker started, entering pull loop");

        // Start reaper as background task
        let reaper_pool = self.pool.clone();
        let reaper_config = self.config.clone();
        let reaper_handle = tokio::spawn(async move {
            Self::run_reaper(reaper_pool, reaper_config).await;
        });

        // Main pull loop
        loop {
            match self.claim_and_execute().await {
                Ok(()) => {
                    // Job executed, continue immediately
                }
                Err(AppError::NoJobsAvailable) => {
                    // Exponential backoff with jitter when queue empty
                    let delay = self.jittered_backoff();
                    trace!("No jobs available, sleeping {:?}", delay);
                    sleep(delay).await;
                }
                Err(e) => {
                    error!("Worker loop error: {:?}", e);
                    // Brief pause on unexpected errors
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Reaper loop - expires stale jobs every 30 seconds
    async fn run_reaper(pool: Arc<PgPool>, config: WorkerConfig) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            match reap_stale_jobs(&pool, MAX_RETRIES).await {
                Ok(reaped) => {
                    if !reaped.is_empty() {
                        info!("Reaper expired {} stale jobs", reaped.len());
                        for job in &reaped {
                            info!("  Reaped job {} (attempt {})", job.id, job.attempts);
                        }
                    }
                }
                Err(e) => {
                    error!("Reaper error: {:?}", e);
                }
            }
        }
    }

    /// Claim a job and execute it
    #[instrument(skip(self))]
    async fn claim_and_execute(&self) -> Result<(), AppError> {
        // 1. CLAIM - atomic transaction with SKIP LOCKED
        let job = claim_job(&self.pool, self.config.lease_seconds).await?;
        
        let job = match job {
            Some(job) => job,
            None => return Err(AppError::NoJobsAvailable),
        };

        info!("Claimed job {} (tool: {:?}, attempt {})", job.id, job.tool, job.attempts);

        // 2. Get semaphore for tool class
        let _permit = self.acquire_tool_permit(&job.tool).await?;

        // 3. MARK RUNNING
        let job = pipeline_core::queue::mark_running(&self.pool, job.id).await?;
        info!("Job {} marked as running", job.id);

        // 3.5 STAGE INPUTS — assembly jobs read curated S3 objects, but the
        // tool sandbox runs network-less; pull inputs onto the volume first.
        if let Some(post) = &self.post {
            if let Err(e) = postprocess::stage_job_inputs(&job, post).await {
                warn!("input staging failed for {}: {:?}", job.id, e);
            }
        }

        // 4. EXECUTE - spawn container and run command
        let receipt = self.execute_job(&job).await;

        // 5. REPORT - mark done/failed based on result
        match receipt {
            Ok(receipt) => {
                // Verify expected outputs exist
                // For now, just mark done
                let job = pipeline_core::queue::mark_done(&self.pool, job.id).await?;
                info!("Job {} completed successfully", job.id);

                // Post-process: ingest outputs to S3 + advance run/design state
                if let Some(post) = &self.post {
                    if let Err(e) = postprocess::ingest_design_outputs(&self.pool, post, job.id).await {
                        warn!("post-process ingest failed for {}: {:?}", job.id, e);
                    }
                }
                if let Err(e) = postprocess::advance_run_status(&self.pool, job.run_id).await {
                    warn!("run status advance failed for {}: {:?}", job.run_id, e);
                }
            }
            Err(e) => {
                error!("Job {} failed: {:?}", job.id, e);
                let job = pipeline_core::queue::mark_failed(&self.pool, job.id, &e.to_string(), MAX_RETRIES).await?;
                if let Err(e) = postprocess::advance_run_status(&self.pool, job.run_id).await {
                    warn!("run status advance failed for {}: {:?}", job.run_id, e);
                }
                info!("Job {} marked as failed (attempt {})", job.id, job.attempts);
            }
        }

        Ok(())
    }

    /// Acquire semaphore permit for tool class
    async fn acquire_tool_permit(&self, tool: &JobTool) -> Result<tokio::sync::OwnedSemaphorePermit, AppError> {
        match tool {
            JobTool::CfImageGen => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {}", e)))?),
            JobTool::PhotocraftCli => Ok(self.photocraft_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {}", e)))?),
            JobTool::VectorcraftCli => Ok(self.vectorcraft_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {}", e)))?),
            JobTool::WebhookProcessor => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {}", e)))?),
            JobTool::ProductAssembly => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {}", e)))?),
            JobTool::PublishListing => Ok(self.cf_image_gen_permit.clone().acquire_owned().await.map_err(|e| AppError::Internal(format!("semaphore acquire failed: {}", e)))?),
        }
    }

    /// Execute job by spawning container
    async fn execute_job(
        &self,
        job: &Job,
    ) -> Result<pipeline_worker::exec::ExecutionReceipt, pipeline_worker::exec::ExecutionError> {
        // Delegate to exec module
        pipeline_worker::exec::execute_job(job, &self.pool).await
    }

    /// Jittered backoff when queue is empty
    fn jittered_backoff(&self) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=self.config.poll_jitter_ms);
        Duration::from_millis(self.config.poll_base_ms + jitter)
    }
}

// Receipt/error types live in the library (`pipeline_worker::exec`), not the bin.

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    // Load configuration (same AppConfig as pipeline-core)
    let config = pipeline_core::config::AppConfig::from_env()?;

    // Create database pool (core helper handles retries + AppError conversion)
    let pool: PgPool = pipeline_core::db::create_pool(&config.database.url, config.database.max_connections).await?;
    let pool = Arc::new(pool);

    // Run migrations (shared with pipeline-core)
    pipeline_core::db::run_migrations(&pool).await?;

    // Create and run worker
    let worker = Worker::new(pool, WorkerConfig::default()).await;
    worker.run().await
}

/// Build the S3 client from environment when all required vars exist.
async fn build_s3_client() -> Option<pipeline_core::s3::S3Client> {
    let endpoint = std::env::var("S3_ENDPOINT").ok()?;
    let bucket = std::env::var("S3_BUCKET").ok()?;
    let access = std::env::var("S3_ACCESS_KEY").ok()?;
    let secret = std::env::var("S3_SECRET_KEY").ok()?;
    let region = std::env::var("APP__S3__REGION").unwrap_or_else(|_| "us-east-1".into());
    match pipeline_core::s3::S3Client::new(&endpoint, &bucket, &access, &secret, &region).await {
        Ok(c) => {
            info!("S3 ingest enabled (bucket {})", bucket);
            Some(c)
        }
        Err(e) => {
            warn!("S3 ingest disabled (client build failed: {})", e);
            None
        }
    }
}
