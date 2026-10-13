//! Worker library for digital-product-studio
//!
//! Contains the worker loop, execution logic, and command-spec jail integration.

pub mod exec;
#[path = "loop.rs"]
pub mod worker_loop;

pub use worker_loop::{WorkerLoop, WorkerLoopConfig};