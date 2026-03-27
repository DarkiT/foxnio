//! 中间件模块

pub mod audit;
pub mod auth;
pub mod compression;
pub mod permission;
pub mod telemetry;

pub use audit::{audit_middleware, login_audit, sensitive_audit, AuditConfig};
pub use auth::*;
pub use compression::{
    compression_middleware, decompression_middleware, get_accept_encoding, get_content_encoding,
    should_compress, CompressedResponse, CompressionLayer, CompressionLevel, CompressionStats,
    ContentEncoding, StatsSnapshot, StreamingCompressor,
};
pub use permission::{
    check_all_permissions, check_any_permission, check_permission, get_permission_service,
    permission_denied, require_admin, require_all_permissions_middleware,
    require_any_permission_middleware, require_manager, require_permission_middleware,
    require_role_middleware, role_denied, with_permission, with_role,
};
pub use telemetry::*;
