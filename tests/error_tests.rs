// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use agentsync::error::{AgentSyncError, Result};

#[test]
fn test_error_types() {
    // Test that all error types can be created
    let io_err = AgentSyncError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
    ));
    assert!(io_err.to_string().contains("file not found"));

    let not_init = AgentSyncError::NotInitialized;
    assert!(not_init.to_string().contains("Project not initialized"));
    assert!(not_init.to_string().contains("agentsync init"));

    let invalid_tool = AgentSyncError::InvalidTool {
        tool: "unknown".to_string(),
    };
    assert!(invalid_tool.to_string().contains("Invalid tool name"));
    assert!(invalid_tool.to_string().contains("Valid tools are"));
}

#[test]
fn test_result_type() {
    fn returns_result(should_fail: bool) -> Result<i32> {
        if should_fail {
            Err(AgentSyncError::NotInitialized)
        } else {
            Ok(42)
        }
    }

    assert_eq!(returns_result(false).unwrap(), 42);
    assert!(returns_result(true).is_err());
}
