//! Injection jail implementation for command-spec validation
//!
//! Implements the 4-vector injection jail from 6.2-command-spec-jail-spec:
//! 1. --params JSON string (shell injection) -> --params-file
//! 2. Output path (path traversal) -> canonicalize under /workspace/write
//! 3. Input path (path traversal/symlink) -> canonicalize under /workspace/read
//! 4. Subcommand name (arbitrary binary) -> allowlist

use crate::jobspec::{JobCommandSpec, ValidatedJobTool};
use crate::error::AppError;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JailError {
    #[error("Root schema validation failed: {0}")]
    RootSchema(String),
    #[error("Subcommand not allowed: {0}")]
    Subcommand(String),
    #[error("Tool args validation failed: {0}")]
    ToolArgs(String),
    #[error("Expected output path validation failed: {0}")]
    ExpectedOutput(String),
    #[error("Path confinement violation: {0}")]
    PathConfinement(String),
    #[error("Shell metacharacter detected: {0}")]
    ShellMetacharacter(String),
}

/// Prepared job after jail validation
#[derive(Debug, Clone)]
pub struct PreparedJob {
    pub job_id: uuid::Uuid,
    pub tool: ValidatedJobTool,
    pub subcommand: String,
    pub args: Value,
    pub timeout_secs: u32,
    pub expected_outputs: Vec<String>,
    pub params_file_path: Option<PathBuf>,
    pub command: Vec<String>,
}

/// Validate and prepare a job spec through the full jail pipeline
pub fn validate_and_prepare(
    spec: &JobCommandSpec,
    job_id: &uuid::Uuid,
) -> Result<PreparedJob, JailError> {
    // 1. Validate root-level schema
    validate_root_schema(spec)?;
    
    // 2. Validate subcommand against allowlist (Vector 4)
    validate_subcommand(spec)?;
    
    // 3. Validate tool-specific args against schemars schema
    validate_tool_args(spec)?;
    
    // 4. Validate expected outputs (must be under /workspace/write)
    for output in &spec.expected_outputs {
        validate_output_path(output)?;
    }
    
    // 5. Build validated argument vector (G2 - zero shell interpolation)
    let command = build_validated_command(spec, job_id)?;
    
    // 6. Handle params file for complex args (Vector 1 mitigation)
    let params_file_path = if spec.args.as_object().map(|o| o.len()).unwrap_or(0) > 3 {
        let params_dir = PathBuf::from("/workspace/params");
        Some(params_dir.join(format!("job-{}.json", job_id)))
    } else {
        None
    };
    
    Ok(PreparedJob {
        job_id: *job_id,
        tool: spec.tool,
        subcommand: spec.subcommand.clone(),
        args: spec.args.clone(),
        timeout_secs: spec.timeout_secs,
        expected_outputs: spec.expected_outputs.clone(),
        params_file_path,
        command,
    })
}

/// Validate root-level JobCommandSpec schema
fn validate_root_schema(spec: &JobCommandSpec) -> Result<(), JailError> {
    // Tool must be one of the allowed tools
    if !matches!(
        spec.tool,
        ValidatedJobTool::CfImageGen
            | ValidatedJobTool::PhotocraftCli
            | ValidatedJobTool::VectorcraftCli
            | ValidatedJobTool::WebhookProcessor
            | ValidatedJobTool::ProductAssembly
            | ValidatedJobTool::PublishListing
    ) {
        return Err(JailError::RootSchema(format!("Unknown tool: {:?}", spec.tool)));
    }
    
    // Subcommand must be present
    if spec.subcommand.is_empty() {
        return Err(JailError::RootSchema("Subcommand is required".to_string()));
    }
    
    // Timeout bounds
    if spec.timeout_secs < 1 || spec.timeout_secs > 1800 {
        return Err(JailError::RootSchema("timeout_secs must be between 1 and 1800".to_string()));
    }
    
    // Args must be a JSON object
    if !spec.args.is_object() {
        return Err(JailError::RootSchema("args must be a JSON object".to_string()));
    }
    
    // Validate expected outputs for path safety
    for output in &spec.expected_outputs {
        validate_output_path(output)?;
    }
    
    Ok(())
}

/// Validate subcommand against tool-specific allowlist (Vector 4)
fn validate_subcommand(spec: &JobCommandSpec) -> Result<(), JailError> {
    let allowed = get_subcommand_allowlist(&spec.tool);
    
    if !allowed.contains(&spec.subcommand.as_str()) {
        return Err(JailError::Subcommand(format!(
            "Subcommand '{}' not allowed for tool {:?}. Allowed: {:?}",
            spec.subcommand, spec.tool, allowed
        )));
    }
    
    Ok(())
}

/// Get subcommand allowlist for a tool (from 6.2-command-spec-jail-spec)
fn get_subcommand_allowlist(tool: &ValidatedJobTool) -> Vec<&'static str> {
    match tool {
        ValidatedJobTool::CfImageGen => vec!["run", "batch", "info"],
        ValidatedJobTool::PhotocraftCli => vec!["run", "batch", "convert", "info", "commands", "mcp", "serve", "droplet"],
        ValidatedJobTool::VectorcraftCli => vec!["run", "batch", "convert", "info", "commands", "mcp", "serve", "droplet"],
        ValidatedJobTool::WebhookProcessor => vec!["run"],
        ValidatedJobTool::ProductAssembly => vec!["run"],
        ValidatedJobTool::PublishListing => vec!["run", "publish"],
    }
}

/// Validate tool-specific args against schemars-derived schema
fn validate_tool_args(spec: &JobCommandSpec) -> Result<(), JailError> {
    // In production, this would use jsonschema crate with schemars-generated schemas
    // For now, do basic validation
    
    let obj = spec.args.as_object().ok_or_else(|| {
        JailError::ToolArgs("args must be a JSON object".to_string())
    })?;
    
    // Tool-specific validation
    match spec.tool {
        ValidatedJobTool::CfImageGen => validate_cf_image_gen_args(obj),
        ValidatedJobTool::PhotocraftCli => validate_photocraft_args(obj),
        ValidatedJobTool::VectorcraftCli => validate_vectorcraft_args(obj),
        ValidatedJobTool::ProductAssembly => validate_product_assembly_args(obj),
        // webhook-processor / publish-listing carry free-form payloads; the
        // global key/metachar checks above already constrain them.
        ValidatedJobTool::WebhookProcessor | ValidatedJobTool::PublishListing => Ok(()),
    }
}

/// product-assembly args validation: curated input key must be declared.
fn validate_product_assembly_args(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), JailError> {
    if !obj.contains_key("curated_key") {
        return Err(JailError::ToolArgs("product-assembly requires 'curated_key' field".to_string()));
    }
    Ok(())
}

/// cf-image-gen args validation
fn validate_cf_image_gen_args(args: &serde_json::Map<String, Value>) -> Result<(), JailError> {
    // Required: prompt
    if !args.contains_key("prompt") {
        return Err(JailError::ToolArgs("cf-image-gen requires 'prompt' field".to_string()));
    }
    
    // Validate optional fields if present
    if let Some(width) = args.get("width").and_then(|v| v.as_u64()) {
        if width < 64 || width > 2048 {
            return Err(JailError::ToolArgs("cf-image-gen width must be 64-2048".to_string()));
        }
    }
    
    if let Some(height) = args.get("height").and_then(|v| v.as_u64()) {
        if height < 64 || height > 2048 {
            return Err(JailError::ToolArgs("cf-image-gen height must be 64-2048".to_string()));
        }
    }
    
    if let Some(steps) = args.get("steps").and_then(|v| v.as_u64()) {
        if steps < 1 || steps > 100 {
            return Err(JailError::ToolArgs("cf-image-gen steps must be 1-100".to_string()));
        }
    }
    
    if let Some(guidance) = args.get("guidance_scale").and_then(|v| v.as_f64()) {
        if guidance < 1.0 || guidance > 20.0 {
            return Err(JailError::ToolArgs("cf-image-gen guidance_scale must be 1.0-20.0".to_string()));
        }
    }
    
    if let Some(format) = args.get("format").and_then(|v| v.as_str()) {
        if !matches!(format, "png" | "jpg" | "webp") {
            return Err(JailError::ToolArgs("cf-image-gen format must be png, jpg, or webp".to_string()));
        }
    }
    
    Ok(())
}

/// photocraft-cli args validation
fn validate_photocraft_args(args: &serde_json::Map<String, Value>) -> Result<(), JailError> {
    // Required: input, operation, out
    if !args.contains_key("input") {
        return Err(JailError::ToolArgs("photocraft-cli requires 'input' field".to_string()));
    }
    if !args.contains_key("operation") {
        return Err(JailError::ToolArgs("photocraft-cli requires 'operation' field".to_string()));
    }
    if !args.contains_key("out") {
        return Err(JailError::ToolArgs("photocraft-cli requires 'out' field".to_string()));
    }
    
    // Validate operation
    if let Some(op) = args.get("operation").and_then(|v| v.as_str()) {
        if !matches!(op, "extract" | "composite" | "resize" | "convert" | "batch") {
            return Err(JailError::ToolArgs("photocraft-cli operation must be extract|composite|resize|convert|batch".to_string()));
        }
    }
    
    // Validate quality
    if let Some(quality) = args.get("quality").and_then(|v| v.as_u64()) {
        if quality < 1 || quality > 100 {
            return Err(JailError::ToolArgs("photocraft-cli quality must be 1-100".to_string()));
        }
    }
    
    Ok(())
}

/// vectorcraft-cli args validation
fn validate_vectorcraft_args(args: &serde_json::Map<String, Value>) -> Result<(), JailError> {
    // Required: input, operation, out
    if !args.contains_key("input") {
        return Err(JailError::ToolArgs("vectorcraft-cli requires 'input' field".to_string()));
    }
    if !args.contains_key("operation") {
        return Err(JailError::ToolArgs("vectorcraft-cli requires 'operation' field".to_string()));
    }
    if !args.contains_key("out") {
        return Err(JailError::ToolArgs("vectorcraft-cli requires 'out' field".to_string()));
    }
    
    // Validate operation
    if let Some(op) = args.get("operation").and_then(|v| v.as_str()) {
        if !matches!(op, "trace" | "simplify" | "convert" | "batch") {
            return Err(JailError::ToolArgs("vectorcraft-cli operation must be trace|simplify|convert|batch".to_string()));
        }
    }
    
    // Validate threshold
    if let Some(threshold) = args.get("threshold").and_then(|v| v.as_f64()) {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(JailError::ToolArgs("vectorcraft-cli threshold must be 0.0-1.0".to_string()));
        }
    }
    
    // Validate max_colors
    if let Some(max_colors) = args.get("max_colors").and_then(|v| v.as_u64()) {
        if max_colors < 2 || max_colors > 256 {
            return Err(JailError::ToolArgs("vectorcraft-cli max_colors must be 2-256".to_string()));
        }
    }
    
    // Validate format
    if let Some(format) = args.get("format").and_then(|v| v.as_str()) {
        if !matches!(format, "svg" | "pdf" | "eps") {
            return Err(JailError::ToolArgs("vectorcraft-cli format must be svg, pdf, or eps".to_string()));
        }
    }
    
    Ok(())
}

/// Validate expected output path safety (Vector 2)
fn validate_output_path(output: &str) -> Result<(), JailError> {
    let path = Path::new(output);
    
    // Must be relative
    if path.is_absolute() {
        return Err(JailError::ExpectedOutput(format!(
            "Absolute path not allowed in expected_outputs: {}", output
        )));
    }
    
    // No path traversal
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(JailError::ExpectedOutput(format!(
                    "Path traversal (..) not allowed in expected_outputs: {}", output
                )));
            }
            std::path::Component::CurDir => {
                return Err(JailError::ExpectedOutput(format!(
                    "Current directory (.) not allowed in expected_outputs: {}", output
                )));
            }
            _ => {}
        }
    }
    
    // No shell metacharacters
    if output.contains(|c: char| c.is_control() || "`$&|;()<>{}[]*?\\'\"".contains(c)) {
        return Err(JailError::ExpectedOutput(format!(
            "Shell metacharacters not allowed in expected_outputs: {}", output
        )));
    }
    
    Ok(())
}

/// Build validated command vector with zero shell interpolation (G2)
fn build_validated_command(spec: &JobCommandSpec, job_id: &uuid::Uuid) -> Result<Vec<String>, JailError> {
    let mut cmd = Vec::new();
    
    // Binary name — use the serde kebab-case name (matches the published image/binary name)
    let tool_name = serde_json::to_value(spec.tool)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", spec.tool).to_lowercase());
    cmd.push(tool_name);
    
    // Subcommand
    cmd.push(spec.subcommand.clone());
    
    // Args as --key value pairs (JSON values serialized, no shell)
    if let Some(obj) = spec.args.as_object() {
        for (key, value) in obj {
            // Validate key for shell safety
            if key.contains(|c: char| c.is_control() || "`$&|;()<>{}[]*?\\'\"".contains(c)) {
                return Err(JailError::ShellMetacharacter(format!(
                    "Shell metacharacter in arg key: {}", key
                )));
            }
            cmd.push(format!("--{}", key));
            cmd.push(serde_json::to_string(value).map_err(|e| {
                JailError::ToolArgs(format!("Failed to serialize arg {}: {}", key, e))
            })?);
        }
    }
    
    // Add params-file if complex args (Vector 1 mitigation)
    if spec.args.as_object().map(|o| o.len()).unwrap_or(0) > 3 {
        cmd.push("--params-file".to_string());
        cmd.push(format!("/workspace/params/job-{}.json", job_id));
    }
    
    // Add confinement roots (Vector 2, 3 mitigation)
    cmd.push("--automation-read-root".to_string());
    cmd.push("/workspace/read".to_string());
    cmd.push("--automation-write-root".to_string());
    cmd.push("/workspace/write".to_string());
    
    // Expected outputs
    for output in &spec.expected_outputs {
        cmd.push("--out".to_string());
        cmd.push(output.clone());
    }
    
    // Final validation: no shell metacharacters in any argument
    for arg in &cmd {
        if arg.contains(|c: char| [';', '&', '|', '$', '`', '>', '<', '(', ')', '{', '}', '\n', '\r'].contains(&c)) {
            return Err(JailError::ShellMetacharacter(format!(
                "Shell metacharacter in command argument: {}", arg
            )));
        }
    }
    
    Ok(cmd)
}

/// Validate path confinement (input/output under read/write roots)
pub fn validate_path_confinement(
    path: &str,
    is_input: bool,
    read_root: &Path,
    write_root: &Path,
) -> Result<PathBuf, JailError> {
    let requested = Path::new(path);
    let root = if is_input { read_root } else { write_root };
    
    // Canonicalize root
    let canonical_root = root.canonicalize().map_err(|_| {
        JailError::PathConfinement(format!("Cannot canonicalize root: {}", root.display()))
    })?;
    
    // Join with root if relative
    let full_path = if requested.is_relative() {
        canonical_root.join(requested)
    } else {
        requested.to_path_buf()
    };
    
    // Canonicalize full path
    let canonical_full = full_path.canonicalize().map_err(|_| {
        JailError::PathConfinement(format!("Path does not exist or cannot be resolved: {}", full_path.display()))
    })?;
    
    // Verify under root
    if !canonical_full.starts_with(&canonical_root) {
        return Err(JailError::PathConfinement(format!(
            "Path escapes confinement root {}: {}",
            root.display(),
            canonical_full.display()
        )));
    }
    
    Ok(canonical_full)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;
    
    #[test]
    fn test_validate_root_schema_valid() {
        let spec = JobCommandSpec {
            tool: ValidatedJobTool::CfImageGen,
            subcommand: "run".to_string(),
            args: json!({"prompt": "test"}),
            timeout_secs: 300,
            expected_outputs: vec!["output.png".to_string()],
        };
        
        assert!(validate_root_schema(&spec).is_ok());
    }
    
    #[test]
    fn test_validate_root_schema_invalid_tool() {
        // Can't easily test invalid tool since enum restricts it
        // But we can test invalid timeout
        let spec = JobCommandSpec {
            tool: ValidatedJobTool::CfImageGen,
            subcommand: "run".to_string(),
            args: json!({"prompt": "test"}),
            timeout_secs: 0, // Invalid
            expected_outputs: vec![],
        };
        
        assert!(validate_root_schema(&spec).is_err());
    }
    
    #[test]
    fn test_validate_subcommand_allowed() {
        let spec = JobCommandSpec {
            tool: ValidatedJobTool::PhotocraftCli,
            subcommand: "run".to_string(),
            args: json!({"input": "test.psd", "operation": "extract", "out": "test.png"}),
            timeout_secs: 300,
            expected_outputs: vec![],
        };
        
        assert!(validate_subcommand(&spec).is_ok());
    }
    
    #[test]
    fn test_validate_subcommand_not_allowed() {
        let spec = JobCommandSpec {
            tool: ValidatedJobTool::CfImageGen,
            subcommand: "invalid_subcommand".to_string(),
            args: json!({"prompt": "test"}),
            timeout_secs: 300,
            expected_outputs: vec![],
        };
        
        assert!(validate_subcommand(&spec).is_err());
    }
    
    #[test]
    fn test_validate_output_path_traversal() {
        assert!(validate_output_path("../escape.png").is_err());
        assert!(validate_output_path("./relative.png").is_err());
        assert!(validate_output_path("normal/output.png").is_ok());
        assert!(validate_output_path("output.png").is_ok());
    }
    
    #[test]
    fn test_validate_output_path_shell_metachars() {
        assert!(validate_output_path("output;rm -rf /").is_err());
        assert!(validate_output_path("output$(cmd)").is_err());
        assert!(validate_output_path("output`cmd`").is_err());
    }
    
    #[test]
    fn test_build_validated_command_no_shell() {
        let spec = JobCommandSpec {
            tool: ValidatedJobTool::CfImageGen,
            subcommand: "run".to_string(),
            args: json!({"prompt": "test", "width": 512}),
            timeout_secs: 300,
            expected_outputs: vec!["output.png".to_string()],
        };
        
        let job_id = Uuid::new_v4();
        let cmd = build_validated_command(&spec, &job_id).unwrap();
        
        // Should be Vec<String> with no shell metacharacters
        for arg in &cmd {
            assert!(!arg.contains(';'));
            assert!(!arg.contains('&'));
            assert!(!arg.contains('|'));
            assert!(!arg.contains('$'));
            assert!(!arg.contains('`'));
        }
        
        // Should contain expected elements
        assert!(cmd.iter().any(|a| a == "cf-image-gen"));
        assert!(cmd.iter().any(|a| a == "run"));
        assert!(cmd.iter().any(|a| a == "--prompt"));
        assert!(cmd.iter().any(|a| a == "--automation-read-root"));
        assert!(cmd.iter().any(|a| a == "/workspace/read"));
        assert!(cmd.iter().any(|a| a == "--automation-write-root"));
        assert!(cmd.iter().any(|a| a == "/workspace/write"));
    }
}