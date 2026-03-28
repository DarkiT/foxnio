//! WebSocket 支持模块 - OpenAI Responses API v1/v2 协议
//!
//! 基于 Codex CLI WebSocket 实现模式，提供：
//! - WebSocket 服务器端点
//! - 连接池管理
//! - 消息队列
//! - 心跳检测
//! - OpenAI Responses API v1/v2 协议支持

#![allow(dead_code)]
pub mod handler;
pub mod pool;

use serde::{Deserialize, Serialize};
use std::fmt;

/// WebSocket 协议版本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WSProtocol {
    /// OpenAI Realtime API v1 - /v1/realtime
    OpenAIV1,
    /// OpenAI Responses API v2 - /v1/responses
    OpenAIV2,
}

impl fmt::Display for WSProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WSProtocol::OpenAIV1 => write!(f, "openai_v1"),
            WSProtocol::OpenAIV2 => write!(f, "openai_v2"),
        }
    }
}

/// WebSocket 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSMessage {
    /// 事件类型 (e.g., "session.created", "response.created")
    pub event_type: String,
    /// 事件 ID
    pub event_id: String,
    /// 事件数据
    pub data: serde_json::Value,
}

impl WSMessage {
    /// 创建新消息
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        let event_id = uuid::Uuid::new_v4().to_string();
        Self {
            event_type: event_type.into(),
            event_id,
            data,
        }
    }

    /// 从 JSON 解析消息
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// WebSocket 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WSConnectionState {
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 空闲
    Idle,
    /// 忙碌（正在处理请求）
    Busy,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
}

impl fmt::Display for WSConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WSConnectionState::Connecting => write!(f, "connecting"),
            WSConnectionState::Connected => write!(f, "connected"),
            WSConnectionState::Idle => write!(f, "idle"),
            WSConnectionState::Busy => write!(f, "busy"),
            WSConnectionState::Closing => write!(f, "closing"),
            WSConnectionState::Closed => write!(f, "closed"),
        }
    }
}

/// WebSocket 错误类型
#[derive(Debug, thiserror::Error)]
pub enum WSError {
    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("消息解析错误: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("连接池已满")]
    PoolFull,

    #[error("连接不存在: {0}")]
    ConnectionNotFound(String),

    #[error("连接已关闭")]
    ConnectionClosed,

    #[error("超时: {0}")]
    Timeout(String),

    #[error("协议错误: {0}")]
    ProtocolError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("WebSocket 错误: {0}")]
    WsError(String),
}

/// 连接配置
#[derive(Debug, Clone)]
pub struct WSConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 最小空闲连接数
    pub min_idle: usize,
    /// 最大空闲连接数
    pub max_idle: usize,
    /// 连接最大存活时间（秒）
    pub max_age_seconds: u64,
    /// 空闲超时（秒）
    pub idle_timeout_seconds: u64,
    /// 心跳间隔（秒）
    pub heartbeat_interval_seconds: u64,
    /// 每个连接的最大等待队列长度
    pub queue_limit_per_conn: usize,
    /// 连接超时（秒）
    pub dial_timeout_seconds: u64,
    /// 目标利用率
    pub target_utilization: f64,
}

impl Default for WSConfig {
    fn default() -> Self {
        Self {
            max_connections: 8,
            min_idle: 0,
            max_idle: 4,
            max_age_seconds: 3600,          // 1 hour
            idle_timeout_seconds: 90,       // 90 seconds
            heartbeat_interval_seconds: 30, // 30 seconds
            queue_limit_per_conn: 256,
            dial_timeout_seconds: 10,
            target_utilization: 0.7,
        }
    }
}

/// 上游传输协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamTransport {
    /// HTTP SSE (传统方式)
    HttpSse,
    /// Responses WebSocket v1
    ResponsesWebSocket,
    /// Responses WebSocket v2
    ResponsesWebSocketV2,
}

/// 协议决策结果
#[derive(Debug, Clone)]
pub struct ProtocolDecision {
    /// 选择的传输协议
    pub transport: UpstreamTransport,
    /// 决策原因
    pub reason: String,
}

/// 重新导出常用类型
pub use handler::create_handler;
