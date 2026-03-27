//! 钉钉机器人告警通道

use async_trait::async_trait;
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;

use super::{AlertChannel, AlertSendResult, DingTalkChannelConfig};
use crate::alert::{Alert, AlertChannelType};

type HmacSha256 = Hmac<Sha256>;

/// 钉钉告警通道
pub struct DingTalkChannel {
    config: DingTalkChannelConfig,
    client: Client,
    name: String,
}

impl DingTalkChannel {
    pub fn new(config: DingTalkChannelConfig) -> Self {
        let name = "DingTalk".to_string();
        let client = Client::new();
        Self {
            config,
            client,
            name,
        }
    }

    /// 生成签名字符串
    fn generate_sign(&self, timestamp: i64) -> Option<String> {
        let secret = self.config.secret.as_ref()?;

        let string_to_sign = format!("{}\n{}", timestamp, secret);

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
        mac.update(string_to_sign.as_bytes());
        let hmac_result = mac.finalize();

        let sign = base64::engine::general_purpose::STANDARD.encode(hmac_result.into_bytes());
        Some(urlencoding::encode(&sign).to_string())
    }

    /// 构建请求 URL（带签名）
    fn build_url(&self) -> String {
        let mut url = self.config.webhook_url.clone();

        if let Some(_) = self.config.secret {
            let timestamp = chrono::Utc::now().timestamp_millis();
            if let Some(sign) = self.generate_sign(timestamp) {
                let separator = if url.contains('?') { "&" } else { "?" };
                url = format!("{}{}timestamp={}&sign={}", url, separator, timestamp, sign);
            }
        }

        url
    }

    /// 构建 Markdown 消息
    fn build_markdown_message(&self, alert: &Alert) -> serde_json::Value {
        let level_emoji = alert.level.icon();
        let level_text = alert.level.as_str().to_uppercase();

        // 构建消息内容
        let mut content = format!(
            "## {} {}\n\n**{}**\n\n> 来源: {}\n> 时间: {}\n\n{}\n",
            level_emoji,
            alert.title,
            level_text,
            alert.source,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.message
        );

        // 添加标签
        if !alert.labels.is_empty() {
            content.push_str("\n**标签:**\n");
            for (key, value) in &alert.labels {
                content.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        // 添加 @ 信息
        let mut at_mobiles = self.config.at_mobiles.clone();
        if self.config.at_all {
            at_mobiles.push("all".to_string());
        }

        serde_json::json!({
            "msgtype": "markdown",
            "markdown": {
                "title": format!("{} {}", level_emoji, alert.title),
                "text": content
            },
            "at": {
                "atMobiles": at_mobiles,
                "isAtAll": self.config.at_all
            }
        })
    }

    /// 构建文本消息
    fn build_text_message(&self, alert: &Alert) -> serde_json::Value {
        let content = alert.to_detailed();

        let mut at_mobiles = self.config.at_mobiles.clone();
        let mut text = content;

        if self.config.at_all {
            text.push_str("\n@所有人");
        } else if !self.config.at_mobiles.is_empty() {
            for mobile in &self.config.at_mobiles {
                text.push_str(&format!("@{} ", mobile));
            }
        }

        serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": text
            },
            "at": {
                "atMobiles": at_mobiles,
                "isAtAll": self.config.at_all
            }
        })
    }
}

#[async_trait]
impl AlertChannel for DingTalkChannel {
    async fn send(&self, alert: &Alert) -> AlertSendResult {
        let url = self.build_url();
        let body = self.build_markdown_message(alert);

        match self.client.post(&url).json(&body).send().await {
            Ok(response) => match response.json::<DingTalkResponse>().await {
                Ok(resp) => {
                    if resp.errcode == 0 {
                        AlertSendResult::success(AlertChannelType::DingTalk)
                    } else {
                        AlertSendResult::failure(
                            AlertChannelType::DingTalk,
                            format!("Error {}: {}", resp.errcode, resp.errmsg),
                        )
                    }
                }
                Err(e) => AlertSendResult::failure(
                    AlertChannelType::DingTalk,
                    format!("Failed to parse response: {}", e),
                ),
            },
            Err(e) => AlertSendResult::failure(
                AlertChannelType::DingTalk,
                format!("Request failed: {}", e),
            ),
        }
    }

    fn channel_type(&self) -> AlertChannelType {
        AlertChannelType::DingTalk
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        !self.config.webhook_url.is_empty()
    }
}

/// 钉钉响应
#[derive(Debug, serde::Deserialize)]
struct DingTalkResponse {
    errcode: i64,
    errmsg: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alert::AlertLevel;

    fn create_test_config() -> DingTalkChannelConfig {
        DingTalkChannelConfig {
            webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=test".to_string(),
            secret: Some("test_secret".to_string()),
            at_mobiles: vec!["13800138000".to_string()],
            at_all: false,
        }
    }

    fn create_test_alert() -> Alert {
        Alert::new(AlertLevel::Critical, "系统故障", "数据库连接失败")
            .with_source("database")
            .with_label("server", "db-01")
    }

    #[test]
    fn test_dingtalk_channel_creation() {
        let config = create_test_config();
        let channel = DingTalkChannel::new(config);

        assert_eq!(channel.channel_type(), AlertChannelType::DingTalk);
        assert!(channel.is_available());
    }

    #[test]
    fn test_build_markdown_message() {
        let config = create_test_config();
        let channel = DingTalkChannel::new(config);
        let alert = create_test_alert();

        let body = channel.build_markdown_message(&alert);

        assert_eq!(body["msgtype"], "markdown");
        assert!(body["markdown"]["title"]
            .as_str()
            .unwrap()
            .contains("系统故障"));
    }

    #[test]
    fn test_build_text_message() {
        let config = create_test_config();
        let channel = DingTalkChannel::new(config);
        let alert = create_test_alert();

        let body = channel.build_text_message(&alert);

        assert_eq!(body["msgtype"], "text");
        assert!(body["text"]["content"]
            .as_str()
            .unwrap()
            .contains("数据库连接失败"));
    }

    #[test]
    fn test_generate_sign() {
        let config = create_test_config();
        let channel = DingTalkChannel::new(config);

        let timestamp = 1700000000000i64;
        let sign = channel.generate_sign(timestamp);

        assert!(sign.is_some());
        assert!(!sign.unwrap().is_empty());
    }

    #[test]
    fn test_build_url_with_sign() {
        let config = create_test_config();
        let channel = DingTalkChannel::new(config);

        let url = channel.build_url();

        assert!(url.contains("timestamp="));
        assert!(url.contains("sign="));
    }

    #[test]
    fn test_at_all() {
        let mut config = create_test_config();
        config.at_all = true;
        let channel = DingTalkChannel::new(config);
        let alert = create_test_alert();

        let body = channel.build_markdown_message(&alert);

        assert_eq!(body["at"]["isAtAll"], true);
    }

    #[test]
    fn test_channel_not_available() {
        let config = DingTalkChannelConfig {
            webhook_url: "".to_string(),
            secret: None,
            at_mobiles: vec![],
            at_all: false,
        };

        let channel = DingTalkChannel::new(config);
        assert!(!channel.is_available());
    }
}
