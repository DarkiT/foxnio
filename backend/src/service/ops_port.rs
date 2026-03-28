use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Port trait for operations service
#[async_trait]
pub trait OpsPort: Send + Sync {
    /// Get operation status
    async fn get_status(&self, operation_id: &str) -> Result<OperationStatus, OpsError>;

    /// List operations
    async fn list_operations(&self, filter: OperationFilter) -> Result<Vec<Operation>, OpsError>;

    /// Cancel operation
    async fn cancel_operation(&self, operation_id: &str) -> Result<(), OpsError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub operation_type: String,
    pub status: OperationStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationFilter {
    pub status: Option<OperationStatus>,
    pub operation_type: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    #[error("Operation not found")]
    NotFound,
    #[error("Operation cannot be cancelled")]
    CannotCancel,
    #[error("Database error: {0}")]
    Database(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_filter() {
        let filter = OperationFilter {
            status: Some(OperationStatus::Running),
            operation_type: None,
            limit: Some(10),
            offset: None,
        };

        assert_eq!(filter.limit, Some(10));
    }
}
