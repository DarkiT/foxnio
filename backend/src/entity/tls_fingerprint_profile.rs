//! TLS 指纹配置模板实体
//!
//! 用于模拟特定客户端（如 Claude Code / Node.js）的 TLS 握手特征

#![allow(dead_code)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// TLS 指纹配置模板
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tls_fingerprint_profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// 模板名称（唯一）
    pub name: String,

    /// 模板描述
    pub description: Option<String>,

    /// 是否启用 GREASE 扩展（Chrome 使用，Node.js 不使用）
    pub enable_grease: bool,

    /// TLS 加密套件列表（顺序敏感，影响 JA3 指纹）
    #[sea_orm(column_type = "Json")]
    pub cipher_suites: JsonValue,

    /// 椭圆曲线/支持的组列表
    #[sea_orm(column_type = "Json")]
    pub curves: JsonValue,

    /// EC 点格式列表
    #[sea_orm(column_type = "Json")]
    pub point_formats: JsonValue,

    /// 签名算法列表
    #[sea_orm(column_type = "Json")]
    pub signature_algorithms: JsonValue,

    /// ALPN 协议列表（如 ["http/1.1"]）
    #[sea_orm(column_type = "Json")]
    pub alpn_protocols: JsonValue,

    /// 支持的 TLS 版本列表（如 [0x0304, 0x0303]）
    #[sea_orm(column_type = "Json")]
    pub supported_versions: JsonValue,

    /// Key Share 中发送的曲线组（如 [29] 即 X25519）
    #[sea_orm(column_type = "Json")]
    pub key_share_groups: JsonValue,

    /// PSK 密钥交换模式（如 [1] 即 psk_dhe_ke）
    #[sea_orm(column_type = "Json")]
    pub psk_modes: JsonValue,

    /// TLS 扩展类型 ID 列表，按发送顺序排列
    #[sea_orm(column_type = "Json")]
    pub extensions: JsonValue,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 关联关系可以在这里定义
}

impl ActiveModelBehavior for ActiveModel {}

/// 创建 TLS 指纹配置模板的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTLSFingerprintProfileRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub enable_grease: bool,
    #[serde(default)]
    pub cipher_suites: Vec<u16>,
    #[serde(default)]
    pub curves: Vec<u16>,
    #[serde(default)]
    pub point_formats: Vec<u16>,
    #[serde(default)]
    pub signature_algorithms: Vec<u16>,
    #[serde(default)]
    pub alpn_protocols: Vec<String>,
    #[serde(default)]
    pub supported_versions: Vec<u16>,
    #[serde(default)]
    pub key_share_groups: Vec<u16>,
    #[serde(default)]
    pub psk_modes: Vec<u16>,
    #[serde(default)]
    pub extensions: Vec<u16>,
}

/// 更新 TLS 指纹配置模板的请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTLSFingerprintProfileRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enable_grease: Option<bool>,
    pub cipher_suites: Option<Vec<u16>>,
    pub curves: Option<Vec<u16>>,
    pub point_formats: Option<Vec<u16>>,
    pub signature_algorithms: Option<Vec<u16>>,
    pub alpn_protocols: Option<Vec<String>>,
    pub supported_versions: Option<Vec<u16>>,
    pub key_share_groups: Option<Vec<u16>>,
    pub psk_modes: Option<Vec<u16>>,
    pub extensions: Option<Vec<u16>>,
}

// 辅助函数：从 JsonValue 提取 Vec<u16>
impl Model {
    pub fn get_cipher_suites(&self) -> Vec<u16> {
        self.cipher_suites
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u16))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_curves(&self) -> Vec<u16> {
        self.curves
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u16))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_alpn_protocols(&self) -> Vec<String> {
        self.alpn_protocols
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}
