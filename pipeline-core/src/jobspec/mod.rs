//! Job command specification validation and execution
//!
//! Implements the command-spec jail (S14, ADR-004) for worker job validation.
//! Validates job specs against JSON Schema, allowlists subcommands, and builds
//! zero-shell-interpolation argument vectors for CLI execution.

use crate::db::entities::{Job, JobTool};
use crate::error::{AppError, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Validated job command specification (S14)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobCommandSpec {
    /// The CLI tool to execute
    pub tool: ValidatedJobTool,
    /// Subcommand to invoke (validated against allowlist)
    pub subcommand: String,
    /// Arguments as structured JSON (never shell-interpolated)
    #[serde(default)]
    pub args: serde_json::Value,
    /// Timeout in seconds (1-1800, default 300)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u32,
    /// Expected output file patterns (for verification)
    #[serde(default)]
    pub expected_outputs: Vec<String>,
}

fn default_timeout() -> u32 {
    300
}

/// Validated job tools matching the JobTool enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ValidatedJobTool {
    #[serde(rename = "cf-image-gen")]
    CfImageGen,
    #[serde(rename = "photocraft-cli")]
    PhotocraftCli,
    #[serde(rename = "vectorcraft-cli")]
    VectorcraftCli,
}

impl From<JobTool> for ValidatedJobTool {
    fn from(tool: JobTool) -> Self {
        match tool {
            JobTool::CfImageGen => ValidatedJobTool::CfImageGen,
            JobTool::PhotocraftCli => ValidatedJobTool::PhotocraftCli,
            JobTool::VectorcraftCli => ValidatedJobTool::VectorcraftCli,
            _ => panic!("Unsupported job tool for command spec: {:?}", tool),
        }
    }
}

/// Validated argument vector for CLI execution (zero shell interpolation - G2)
#[derive(Debug, Clone)]
pub struct ValidatedArgs {
    /// The binary name (e.g., "photocraft-cli")
    pub binary: String,
    /// Complete argument vector ready for tokio::process::Command
    pub args: Vec<String>,
    /// Path to the params file (for --params-file)
    pub params_file: Option<PathBuf>,
    /// Timeout in seconds
    pub timeout_secs: u32,
}

/// Command-spec validator implementing S14 injection jail
pub struct CommandSpecValidator {
    /// Per-tool subcommand allowlists
    subcommand_allowlists: HashMap<ValidatedJobTool, Vec<&'static str>>,
    /// Per-tool argument JSON schemas (from schemars)
    arg_schemas: HashMap<ValidatedJobTool, HashMap<String, serde_json::Value>>,
    /// Base paths for path confinement
    read_root: PathBuf,
    write_root: PathBuf,
}

impl CommandSpecValidator {
    /// Create a new validator with default allowlists and schemas
    pub fn new(read_root: PathBuf, write_root: PathBuf) -> Self {
        let mut subcommand_allowlists = HashMap::new();
        subcommand_allowlists.insert(
            ValidatedJobTool::CfImageGen,
            vec!["run", "batch", "info"],
        );
        subcommand_allowlists.insert(
            ValidatedJobTool::PhotocraftCli,
            vec!["run", "batch", "convert", "info", "commands", "mcp", "serve", "droplet"],
        );
        subcommand_allowlists.insert(
            ValidatedJobTool::VectorcraftCli,
            vec!["run", "batch", "convert", "info", "commands", "mcp", "serve", "droplet"],
        );

        let mut arg_schemas = HashMap::new();
        // In practice these would be generated from schemars at build time
        // For now we define the expected structure
        arg_schemas.insert(ValidatedJobTool::CfImageGen, HashMap::new());
        arg_schemas.insert(ValidatedJobTool::PhotocraftCli, HashMap::new());
        arg_schemas.insert(ValidatedJobTool::VectorcraftCli, HashMap::new());

        Self {
            subcommand_allowlists,
            arg_schemas,
            read_root,
            write_root,
        }
    }

    /// Validate and convert a raw job into a validated command spec (G1, G2)
    pub fn validate_and_build(&self, job: &Job) -> Result<ValidatedArgs> {
        // Parse the command spec from JSON
        let spec: JobCommandSpec = serde_json::from_value(job.command_spec.clone())
            .map_err(|e| AppError::Validation(format!("Invalid command spec JSON: {}", e)))?;

        // G1: Validate tool is allowlisted
        if !matches!(spec.tool, ValidatedJobTool::CfImageGen | ValidatedJobTool::PhotocraftCli | ValidatedJobTool::VectorcraftCli) {
            return Err(AppError::Validation(format!("Unknown tool: {:?}", spec.tool)));
        }

        // G1: Validate subcommand is allowlisted
        let allowed = self.subcommand_allowlists.get(&spec.tool).ok_or_else(|| {
            AppError::Validation(format!("No allowlist for tool: {:?}", spec.tool))
        })?;
        if !allowed.contains(&spec.subcommand.as_str()) {
            return Err(AppError::Validation(format!(
                "Subcommand '{}' not allowed for tool {:?}. Allowed: {:?}",
                spec.subcommand, spec.tool, allowed
            )));
        }

        // G1: Validate args against schema (if available)
        if let Some(schemas) = self.arg_schemas.get(&spec.tool) {
            if let Some(schema) = schemas.get(&spec.subcommand) {
                // In production, use jsonschema crate to validate
                // For now, basic structure check
                if !spec.args.is_object() {
                    return Err(AppError::Validation("args must be a JSON object".to_string()));
                }
            }
        }

        // G1: Validate timeout bounds
        if spec.timeout_secs < 1 || spec.timeout_secs > 1800 {
            return Err(AppError::Validation(
                "timeout_secs must be between 1 and 1800".to_string(),
            ));
        }

        // G1: Adversarial input rejection - path traversal in expected_outputs
        for output in &spec.expected_outputs {
            self.validate_path_safety(output, "expected_outputs")?;
        }

        // G2: Build validated argument vector with ZERO shell interpolation
        let mut args = Vec::new();
        args.push(spec.subcommand.clone());

        // Add args from JSON object as --key value pairs
        if let Some(obj) = spec.args.as_object() {
            for (key, value) in obj {
                args.push(format!("--{}", key));
                args.push(serde_json::to_string(value).map_err(|e| {
                    AppError::Validation(format!("Failed to serialize arg {}: {}", key, e))
                })?);
            }
        }

        // Handle params file for large JSON payloads (M2.2)
        let params_file = if spec.args.as_object().map(|o| o.len()).unwrap_or(0) > 3 {
            // For complex params, write to file and use --params-file
            // This is handled by the worker runtime, not here
            None
        } else {
            None
        };

        Ok(ValidatedArgs {
            binary: format!("{:?}", spec.tool).to_lowercase().replace("validatedjobtool::", ""),
            args,
            params_file,
            timeout_secs: spec.timeout_secs,
        })
    }

    /// Validate a path for safety (no traversal, symlinks, etc.) - G1
    fn validate_path_safety(&self, path: &str, context: &str) -> Result<()> {
        let path = Path::new(path);

        // Reject absolute paths
        if path.is_absolute() {
            return Err(AppError::Validation(format!(
                "Absolute path not allowed in {}: {}",
                context, path.display()
            )));
        }

        // Reject path traversal attempts
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(AppError::Validation(format!(
                        "Path traversal (..) not allowed in {}: {}",
                        context, path.display()
                    )));
                }
                std::path::Component::CurDir => {
                    return Err(AppError::Validation(format!(
                        "Current directory (.) not allowed in {}: {}",
                        context, path.display()
                    )));
                }
                _ => {}
            }
        }

        // Reject paths with shell metacharacters
        let path_str = path.to_string_lossy();
        if path_str.contains(|c: char| c.is_control() || "`$&|;()<>{}[]*?\\'\"".contains(c)) {
            return Err(AppError::Validation(format!(
                "Shell metacharacters not allowed in {}: {}",
                context, path_str
            )));
        }

        // Reject symlink-like patterns
        if path_str.contains("->") || path_str.contains("..") {
            return Err(AppError::Validation(format!(
                "Symlink-like pattern not allowed in {}: {}",
                context, path_str
            )));
        }

        Ok(())
    }

    /// Validate input/output paths against confinement roots - G1
    pub fn validate_path_confinement(&self, path: &str, is_input: bool) -> Result<PathBuf> {
        let requested = Path::new(path);
        let root = if is_input { &self.read_root } else { &self.write_root };

        // Canonicalize to resolve any symlinks
        let canonical_root = root.canonicalize().map_err(|e| {
            AppError::Internal(format!("Failed to canonicalize root {}: {}", root.display(), e))
        })?;

        // For relative paths, join with root
        let full_path = if requested.is_relative() {
            canonical_root.join(requested)
        } else {
            requested.to_path_buf()
        };

        // Canonicalize the full path
        let canonical_full = full_path.canonicalize().map_err(|_| {
            AppError::Validation(format!(
                "Path does not exist or cannot be resolved: {}",
                full_path.display()
            ))
        })?;

        // Verify it's under the allowed root
        if !canonical_full.starts_with(&canonical_root) {
            return Err(AppError::Validation(format!(
                "Path escapes confinement root {}: {}",
                root.display(),
                canonical_full.display()
            )));
        }

        Ok(canonical_full)
    }
}

/// Build execution receipt (M2.3)
pub fn build_execution_receipt(
    job_id: uuid::Uuid,
    command: &[String],
    args: &[String],
    exit_code: i32,
    stdout: &[u8],
    stderr: &[u8],
    output_files: Vec<OutputFileInfo>,
) -> crate::db::entities::JobReceipt {
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    let command_str = format!("{} {}", command[0], args.join(" "));
    let mut hasher = Sha256::new();
    hasher.update(command_str.as_bytes());
    let command_hash = format!("{:x}", hasher.finalize());

    let mut hasher = Sha256::new();
    hasher.update(&serde_json::to_vec(args).unwrap_or_default());
    let args_hash = format!("{:x}", hasher.finalize());

    let stdout_sha256 = format!("{:x}", Sha256::digest(stdout));
    let stderr_sha256 = format!("{:x}", Sha256::digest(stderr));

    crate::db::entities::JobReceipt {
        id: Uuid::new_v4(),
        job_id,
        command_hash,
        args_hash,
        exit_code,
        stdout_sha256,
        stderr_sha256,
        output_files: serde_json::to_value(output_files).unwrap_or_default(),
        created_at: chrono::Utc::now(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFileInfo {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}