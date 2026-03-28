use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Messages compatibility service for Gemini API
pub struct GeminiMessagesCompatService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiMessage {
    pub role: String,
    pub parts: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "text")]
    pub text: Option<String>,
    #[serde(rename = "inlineData")]
    pub inline_data: Option<InlineData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CompatError {
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Unsupported content type")]
    UnsupportedContentType,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl GeminiMessagesCompatService {
    /// Convert OpenAI messages to Gemini format
    pub fn convert_openai_to_gemini(
        openai_messages: &[Value],
    ) -> Result<Vec<GeminiMessage>, CompatError> {
        let mut gemini_messages = Vec::new();

        for msg in openai_messages {
            let role = msg
                .get("role")
                .and_then(|r| r.as_str())
                .ok_or(CompatError::InvalidFormat)?;

            let content = msg.get("content").ok_or(CompatError::InvalidFormat)?;

            let parts = Self::convert_content_to_parts(content)?;

            let gemini_role = match role {
                "assistant" => "model",
                "user" => "user",
                "system" => "user", // Gemini doesn't have system role
                _ => "user",
            };

            gemini_messages.push(GeminiMessage {
                role: gemini_role.to_string(),
                parts,
            });
        }

        Ok(gemini_messages)
    }

    /// Convert content to Gemini parts
    fn convert_content_to_parts(content: &Value) -> Result<Vec<ContentPart>, CompatError> {
        let mut parts = Vec::new();

        if let Some(text) = content.as_str() {
            parts.push(ContentPart {
                text: Some(text.to_string()),
                inline_data: None,
            });
        } else if let Some(array) = content.as_array() {
            for item in array {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    parts.push(ContentPart {
                        text: Some(text.to_string()),
                        inline_data: None,
                    });
                } else if let Some(image_url) = item.get("image_url") {
                    let url = image_url
                        .get("url")
                        .and_then(|u| u.as_str())
                        .ok_or(CompatError::InvalidFormat)?;

                    // Extract base64 data from data URL
                    if url.starts_with("data:") {
                        let parts_url: Vec<&str> = url.splitn(2, ',').collect();
                        if parts_url.len() == 2 {
                            let mime_type = parts_url[0]
                                .strip_prefix("data:")
                                .and_then(|s| s.strip_suffix(";base64"))
                                .unwrap_or("image/png");

                            parts.push(ContentPart {
                                text: None,
                                inline_data: Some(InlineData {
                                    mime_type: mime_type.to_string(),
                                    data: parts_url[1].to_string(),
                                }),
                            });
                        }
                    }
                }
            }
        }

        Ok(parts)
    }

    /// Convert Gemini response to OpenAI format
    pub fn convert_gemini_to_openai(
        gemini_response: &Value,
        model: &str,
    ) -> Result<Value, CompatError> {
        let text = gemini_response
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|arr| arr.first())
            .and_then(|part| part.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        Ok(serde_json::json!({
            "id": format!("gemini-{}", uuid::Uuid::new_v4()),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": text
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_openai_to_gemini() {
        let openai_messages = vec![serde_json::json!({
            "role": "user",
            "content": "Hello"
        })];

        let gemini_messages =
            GeminiMessagesCompatService::convert_openai_to_gemini(&openai_messages).unwrap();

        assert_eq!(gemini_messages.len(), 1);
        assert_eq!(gemini_messages[0].role, "user");
    }
}
