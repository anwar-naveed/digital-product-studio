//! pipeline-core library
//!
pub mod api;
pub mod audit;
pub mod config;
pub mod db;
pub mod error;
pub mod jail;
pub mod jobspec;
pub mod middleware;
pub mod queue;
pub mod s3;
pub mod state;

pub use error::{AppError, Result};
pub use jail::{validate_and_prepare, PreparedJob};
pub use jobspec::{CommandSpecValidator, ValidatedArgs, build_execution_receipt};
pub use queue::{claim_job, mark_running, mark_done, mark_failed, reap_stale_jobs, DEFAULT_LEASE_SECONDS, MAX_RETRIES, BASE_BACKOFF_SECONDS, QueueStats, ToolConcurrencyStats, get_queue_stats, get_tool_concurrency_stats};
pub use s3::{S3Client, S3ClientFactory, WorkerS3Client, AdminS3Client, S3KeyBuilder, S3Prefix};
pub use state::AppState;