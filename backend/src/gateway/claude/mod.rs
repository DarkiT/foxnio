//! Claude 渠道支持

pub mod constants;
pub mod header_util;
pub mod headers;
pub mod tls;
pub mod tool_use;
pub mod validator;

#[cfg(test)]
mod full_test;

#[allow(unused_imports)]
pub use constants::*;
#[allow(unused_imports)]
pub use header_util::{
    build_claude_headers_ordered, header_wire_casing, sort_headers_by_wire_order,
};
#[allow(unused_imports)]
pub use headers::ClaudeHeaders;
#[allow(unused_imports)]
pub use tls::{TLSFingerprint, DEFAULT_CIPHER_SUITES, DEFAULT_CURVES};
#[allow(unused_imports)]
pub use validator::{parse_metadata_user_id, ClaudeCodeValidator};
