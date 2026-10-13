//! Tests for command-spec validators (G1, G2)

use pipeline_core::db::entities::{Job, JobTool, JobStatus};
use pipeline_core::jobspec::{CommandSpecValidator, ValidatedArgs};
use pipeline_core::error::Result;
use uuid::Uuid;
use chrono::Utc;
use std::path::PathBuf;

fn create_test_job(tool: JobTool, command_spec: serde_json::Value) -> Job {
    Job {
        id: Uuid::new_v4(),
        run_id: Uuid::new_v4(),
        tool,
        command_spec,
        status: JobStatus::Queued,
        attempts: 0,
        queued_at: Utc::now(),
        started_at: None,
        finished_at: None,
        last_error: None,
        next_attempt_at: None,
        lock_expires_at: None,
    }
}

#[cfg(test)]
mod command_spec_validation_tests {
    use super::*;

    #[test]
    fn test_valid_cf_image_gen_spec() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {
                    "prompt": "gothic wall art",
                    "dimensions": "2480x3508"
                },
                "timeout_secs": 300,
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_ok());
        
        let args = result.unwrap();
        assert_eq!(args.binary, "cf-image-gen");
        assert!(args.args.contains(&"run".to_string()));
        assert!(args.args.contains(&"--prompt".to_string()));
        assert!(args.args.contains(&"\"gothic wall art\"".to_string()));
    }

    #[test]
    fn test_valid_photocraft_cli_spec() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::PhotocraftCli,
            serde_json::json!({
                "tool": "photocraft-cli",
                "subcommand": "run",
                "args": {
                    "input": "input.psd",
                    "cmd": "filter.sharpen.smartSharpen",
                    "params": {"amount": 80},
                    "out": "output.png"
                },
                "timeout_secs": 300,
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_ok());
        
        let args = result.unwrap();
        assert_eq!(args.binary, "photocraft-cli");
        assert!(args.args.contains(&"run".to_string()));
    }

    #[test]
    fn test_valid_vectorcraft_cli_spec() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::VectorcraftCli,
            serde_json::json!({
                "tool": "vectorcraft-cli",
                "subcommand": "convert",
                "args": {
                    "input": "input.svg",
                    "out": "output.png",
                    "width": 1024,
                    "height": 1024
                },
                "timeout_secs": 300,
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_ok());
        
        let args = result.unwrap();
        assert_eq!(args.binary, "vectorcraft-cli");
    }

    // G1: Adversarial spec rejection tests

    #[test]
    fn test_reject_path_traversal_in_expected_outputs() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["../../../etc/passwd"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path traversal"));
    }

    #[test]
    fn test_reject_absolute_path_in_expected_outputs() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["/etc/passwd"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Absolute path not allowed"));
    }

    #[test]
    fn test_reject_shell_metacharacters_in_path() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["output; rm -rf /"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Shell metacharacters"));
    }

    #[test]
    fn test_reject_unknown_subcommand() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "malicious",
                "args": {},
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not allowed"));
    }

    #[test]
    fn test_reject_unknown_tool() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::WebhookProcessor, // Not in allowlist for command spec
            serde_json::json!({
                "tool": "webhook-processor",
                "subcommand": "run",
                "args": {},
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_timeout_too_large() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "timeout_secs": 2000, // > 1800 max
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout_secs must be between 1 and 1800"));
    }

    #[test]
    fn test_reject_timeout_zero() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "timeout_secs": 0,
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_symlink_like_pattern() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["output->symlink"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Symlink-like pattern"));
    }

    #[test]
    fn test_reject_current_dir_reference() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["./output.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Current directory"));
    }

    #[test]
    fn test_reject_control_characters() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["output\x00.png"]
            }),
        );

        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Shell metacharacters"));
    }

    #[test]
    fn test_g2_zero_shell_interpolation() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        let job = create_test_job(
            JobTool::PhotocraftCli,
            serde_json::json!({
                "tool": "photocraft-cli",
                "subcommand": "run",
                "args": {
                    "input": "test.psd",
                    "cmd": "filter.sharpen",
                    "params": {"amount": 80}
                },
                "expected_outputs": ["output.png"]
            }),
        );

        let result = validator.validate_and_build(&job).unwrap();
        
        // G2: Verify args vector is built with zero shell interpolation
        // All arguments should be separate elements, no shell metacharacters
        for arg in &result.args {
            assert!(!arg.contains(';'), "Arg contains semicolon: {}", arg);
            assert!(!arg.contains('&'), "Arg contains ampersand: {}", arg);
            assert!(!arg.contains('|'), "Arg contains pipe: {}", arg);
            assert!(!arg.contains('$'), "Arg contains dollar: {}", arg);
            assert!(!arg.contains('`'), "Arg contains backtick: {}", arg);
            assert!(!arg.contains('('), "Arg contains paren: {}", arg);
            assert!(!arg.contains(')'), "Arg contains paren: {}", arg);
        }
        
        // Verify subcommand is first arg
        assert_eq!(result.args[0], "run");
    }

    #[test]
    fn test_path_confinement_validation() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        // Test valid path under read root
        let result = validator.validate_path_confinement("input.psd", true);
        // This will fail because the file doesn't exist, but we can test the logic
        // by checking the error type
        assert!(result.is_err()); // File doesn't exist in test env
        
        // Test path traversal attempt
        let result = validator.validate_path_confinement("../../../etc/passwd", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("escapes confinement"));
    }

    #[test]
    fn test_valid_subcommands_cf_image_gen() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        for subcommand in ["run", "batch", "info"] {
            let job = create_test_job(
                JobTool::CfImageGen,
                serde_json::json!({
                    "tool": "cf-image-gen",
                    "subcommand": subcommand,
                    "args": {"prompt": "test"},
                    "expected_outputs": ["output.png"]
                }),
            );

            let result = validator.validate_and_build(&job);
            assert!(result.is_ok(), "Subcommand '{}' should be allowed", subcommand);
        }
    }

    #[test]
    fn test_valid_subcommands_photocraft_cli() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        for subcommand in ["run", "batch", "convert", "info", "commands", "mcp", "serve", "droplet"] {
            let job = create_test_job(
                JobTool::PhotocraftCli,
                serde_json::json!({
                    "tool": "photocraft-cli",
                    "subcommand": subcommand,
                    "args": {"input": "test.psd"},
                    "expected_outputs": ["output.png"]
                }),
            );

            let result = validator.validate_and_build(&job);
            assert!(result.is_ok(), "Subcommand '{}' should be allowed", subcommand);
        }
    }

    #[test]
    fn test_valid_subcommands_vectorcraft_cli() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        for subcommand in ["run", "batch", "convert", "info", "commands", "mcp", "serve", "droplet"] {
            let job = create_test_job(
                JobTool::VectorcraftCli,
                serde_json::json!({
                    "tool": "vectorcraft-cli",
                    "subcommand": subcommand,
                    "args": {"input": "test.svg"},
                    "expected_outputs": ["output.png"]
                }),
            );

            let result = validator.validate_and_build(&job);
            assert!(result.is_ok(), "Subcommand '{}' should be allowed", subcommand);
        }
    }

    #[test]
    fn test_args_structure_is_json_object() {
        let validator = CommandSpecValidator::new(
            PathBuf::from("/workspace/read"),
            PathBuf::from("/workspace/write"),
        );

        // Valid: args is an object
        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": {"prompt": "test"},
                "expected_outputs": ["output.png"]
            }),
        );
        assert!(validator.validate_and_build(&job).is_ok());

        // Invalid: args is an array
        let job = create_test_job(
            JobTool::CfImageGen,
            serde_json::json!({
                "tool": "cf-image-gen",
                "subcommand": "run",
                "args": ["prompt", "test"],
                "expected_outputs": ["output.png"]
            }),
        );
        let result = validator.validate_and_build(&job);
        assert!(result.is_err());
    }
}