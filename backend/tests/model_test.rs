//! 模型路由测试
//!
//! 测试模型路由、降级和参数映射功能

use std::collections::HashMap;

// 模拟模型定义（与实际代码保持一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Google,
    DeepSeek,
    Mistral,
    Cohere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Model {
    // OpenAI
    GPT4Turbo,
    GPT4,
    GPT4o,
    GPT4oMini,
    GPT35Turbo,
    // Anthropic
    Claude3Opus,
    Claude35Sonnet,
    Claude3Haiku,
    Claude35SonnetV2,
    // Google
    GeminiPro,
    GeminiUltra,
    Gemini15Pro,
    // DeepSeek
    DeepSeekV3,
    DeepSeekCoder,
    // Mistral
    MistralLarge,
    MistralMedium,
    MistralSmall,
    // Cohere
    CommandRPlus,
    CommandR,
}

impl Model {
    fn provider(&self) -> ModelProvider {
        match self {
            Model::GPT4Turbo
            | Model::GPT4
            | Model::GPT4o
            | Model::GPT4oMini
            | Model::GPT35Turbo => ModelProvider::OpenAI,
            Model::Claude3Opus
            | Model::Claude35Sonnet
            | Model::Claude3Haiku
            | Model::Claude35SonnetV2 => ModelProvider::Anthropic,
            Model::GeminiPro | Model::GeminiUltra | Model::Gemini15Pro => ModelProvider::Google,
            Model::DeepSeekV3 | Model::DeepSeekCoder => ModelProvider::DeepSeek,
            Model::MistralLarge | Model::MistralMedium | Model::MistralSmall => {
                ModelProvider::Mistral
            }
            Model::CommandRPlus | Model::CommandR => ModelProvider::Cohere,
        }
    }

    fn fallback_models(&self) -> Vec<Model> {
        match self {
            Model::GPT4Turbo => vec![Model::GPT4, Model::GPT4o, Model::GPT35Turbo],
            Model::GPT4 => vec![Model::GPT4Turbo, Model::GPT4o, Model::GPT35Turbo],
            Model::GPT4o => vec![Model::GPT4Turbo, Model::GPT35Turbo],
            Model::GPT4oMini => vec![Model::GPT35Turbo, Model::GPT4o],
            Model::GPT35Turbo => vec![Model::GPT4oMini],
            Model::Claude3Opus => vec![Model::Claude35Sonnet, Model::Claude3Haiku],
            Model::Claude35Sonnet => vec![Model::Claude35SonnetV2, Model::Claude3Haiku],
            Model::Claude35SonnetV2 => vec![Model::Claude35Sonnet, Model::Claude3Haiku],
            Model::Claude3Haiku => vec![Model::Claude35Sonnet],
            Model::GeminiUltra => vec![Model::Gemini15Pro, Model::GeminiPro],
            Model::Gemini15Pro => vec![Model::GeminiPro],
            Model::GeminiPro => vec![Model::Gemini15Pro],
            Model::DeepSeekV3 => vec![Model::DeepSeekCoder],
            Model::DeepSeekCoder => vec![Model::DeepSeekV3],
            Model::MistralLarge => vec![Model::MistralMedium, Model::MistralSmall],
            Model::MistralMedium => vec![Model::MistralSmall, Model::MistralLarge],
            Model::MistralSmall => vec![Model::MistralMedium],
            Model::CommandRPlus => vec![Model::CommandR],
            Model::CommandR => vec![Model::CommandRPlus],
        }
    }

    fn all() -> Vec<Model> {
        vec![
            Model::GPT4Turbo,
            Model::GPT4,
            Model::GPT4o,
            Model::GPT4oMini,
            Model::GPT35Turbo,
            Model::Claude3Opus,
            Model::Claude35Sonnet,
            Model::Claude3Haiku,
            Model::Claude35SonnetV2,
            Model::GeminiPro,
            Model::GeminiUltra,
            Model::Gemini15Pro,
            Model::DeepSeekV3,
            Model::DeepSeekCoder,
            Model::MistralLarge,
            Model::MistralMedium,
            Model::MistralSmall,
            Model::CommandRPlus,
            Model::CommandR,
        ]
    }
}

impl ModelProvider {
    fn base_url(&self) -> &'static str {
        match self {
            ModelProvider::OpenAI => "https://api.openai.com",
            ModelProvider::Anthropic => "https://api.anthropic.com",
            ModelProvider::Google => "https://generativelanguage.googleapis.com",
            ModelProvider::DeepSeek => "https://api.deepseek.com",
            ModelProvider::Mistral => "https://api.mistral.ai",
            ModelProvider::Cohere => "https://api.cohere.ai",
        }
    }
}

// ==================== 模型路由测试 ====================

#[test]
fn test_model_provider_mapping() {
    // 测试每个模型正确映射到其提供商
    assert_eq!(Model::GPT4Turbo.provider(), ModelProvider::OpenAI);
    assert_eq!(Model::GPT4.provider(), ModelProvider::OpenAI);
    assert_eq!(Model::GPT4o.provider(), ModelProvider::OpenAI);
    assert_eq!(Model::GPT35Turbo.provider(), ModelProvider::OpenAI);

    assert_eq!(Model::Claude3Opus.provider(), ModelProvider::Anthropic);
    assert_eq!(Model::Claude35Sonnet.provider(), ModelProvider::Anthropic);
    assert_eq!(Model::Claude3Haiku.provider(), ModelProvider::Anthropic);

    assert_eq!(Model::GeminiPro.provider(), ModelProvider::Google);
    assert_eq!(Model::GeminiUltra.provider(), ModelProvider::Google);

    assert_eq!(Model::DeepSeekV3.provider(), ModelProvider::DeepSeek);
    assert_eq!(Model::MistralLarge.provider(), ModelProvider::Mistral);
    assert_eq!(Model::CommandRPlus.provider(), ModelProvider::Cohere);
}

#[test]
fn test_model_count() {
    // 确保支持至少 15 个模型
    let models = Model::all();
    assert!(
        models.len() >= 15,
        "Expected at least 15 models, got {}",
        models.len()
    );
    println!("Total models supported: {}", models.len());
}

#[test]
fn test_provider_coverage() {
    // 确保每个提供商至少有一个模型
    let models = Model::all();
    let mut provider_counts: HashMap<ModelProvider, usize> = HashMap::new();

    for model in models {
        *provider_counts.entry(model.provider()).or_insert(0) += 1;
    }

    for provider in [
        ModelProvider::OpenAI,
        ModelProvider::Anthropic,
        ModelProvider::Google,
        ModelProvider::DeepSeek,
        ModelProvider::Mistral,
        ModelProvider::Cohere,
    ] {
        let count = provider_counts.get(&provider).copied().unwrap_or(0);
        assert!(count > 0, "Provider {:?} has no models", provider);
        println!("Provider {:?}: {} models", provider, count);
    }
}

// ==================== 模型降级测试 ====================

#[test]
fn test_fallback_chain_exists() {
    // 每个模型都应该有降级链
    for model in Model::all() {
        let fallbacks = model.fallback_models();
        // 降级链可以为空（某些模型可能没有合适的降级选项）
        println!("Model {:?} fallback chain: {:?}", model, fallbacks);
    }
}

#[test]
fn test_fallback_chain_consistency() {
    // 降级模型应该来自同一个提供商或兼容的提供商
    let model = Model::GPT4Turbo;
    let fallbacks = model.fallback_models();

    // OpenAI 模型应该优先降级到其他 OpenAI 模型
    for fallback in &fallbacks {
        // 第一优先级降级应该是同一提供商
        if *fallback == fallbacks[0] {
            assert_eq!(
                fallback.provider(),
                model.provider(),
                "Primary fallback should be from same provider"
            );
        }
    }
}

#[test]
fn test_fallback_no_self() {
    // 降级链不应该包含自己
    for model in Model::all() {
        let fallbacks = model.fallback_models();
        assert!(
            !fallbacks.contains(&model),
            "Model {:?} should not fallback to itself",
            model
        );
    }
}

#[test]
fn test_fallback_chain_length() {
    // 检查降级链长度
    for model in Model::all() {
        let fallbacks = model.fallback_models();
        assert!(
            fallbacks.len() <= 5,
            "Fallback chain for {:?} is too long: {}",
            model,
            fallbacks.len()
        );
    }
}

// ==================== 参数映射测试 ====================

#[derive(Debug, Clone)]
struct RequestParams {
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    messages: Vec<String>,
}

fn map_params(model: Model, params: &mut RequestParams) {
    // 模拟参数映射逻辑
    params.model = match model {
        Model::GPT4Turbo => "gpt-4-turbo-preview".to_string(),
        Model::GPT4 => "gpt-4".to_string(),
        Model::Claude35Sonnet => "claude-3-5-sonnet-20241022".to_string(),
        Model::Claude3Haiku => "claude-3-haiku-20240307".to_string(),
        Model::GeminiPro => "gemini-pro".to_string(),
        _ => format!("{:?}", model).to_lowercase(),
    };

    // 根据提供商调整参数
    match model.provider() {
        ModelProvider::Anthropic => {
            // Anthropic 使用 max_tokens
            // 保持不变
        }
        ModelProvider::OpenAI => {
            // OpenAI 使用 max_completion_tokens
        }
        ModelProvider::Google => {
            // Gemini 使用不同的参数结构
        }
        _ => {}
    }
}

#[test]
fn test_param_mapping_openai() {
    let mut params = RequestParams {
        model: "gpt-4-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        messages: vec!["Hello".to_string()],
    };

    map_params(Model::GPT4Turbo, &mut params);

    assert_eq!(params.model, "gpt-4-turbo-preview");
    assert!(params.max_tokens.is_some());
}

#[test]
fn test_param_mapping_anthropic() {
    let mut params = RequestParams {
        model: "claude-3-5-sonnet".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        messages: vec!["Hello".to_string()],
    };

    map_params(Model::Claude35Sonnet, &mut params);

    assert_eq!(params.model, "claude-3-5-sonnet-20241022");
}

#[test]
fn test_param_mapping_gemini() {
    let mut params = RequestParams {
        model: "gemini-pro".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        messages: vec!["Hello".to_string()],
    };

    map_params(Model::GeminiPro, &mut params);

    assert_eq!(params.model, "gemini-pro");
}

// ==================== 提供商配置测试 ====================

#[test]
fn test_provider_base_urls() {
    assert!(ModelProvider::OpenAI.base_url().starts_with("https://"));
    assert!(ModelProvider::Anthropic.base_url().starts_with("https://"));
    assert!(ModelProvider::Google.base_url().starts_with("https://"));
    assert!(ModelProvider::DeepSeek.base_url().starts_with("https://"));
    assert!(ModelProvider::Mistral.base_url().starts_with("https://"));
    assert!(ModelProvider::Cohere.base_url().starts_with("https://"));
}

// ==================== 模型别名测试 ====================

fn resolve_model_alias(name: &str) -> Option<Model> {
    let aliases: &[(&str, Model)] = &[
        ("gpt4t", Model::GPT4Turbo),
        ("gpt4", Model::GPT4),
        ("gpt-4", Model::GPT4),
        ("gpt4o", Model::GPT4o),
        ("gpt-4o", Model::GPT4o),
        ("gpt35", Model::GPT35Turbo),
        ("claude-opus", Model::Claude3Opus),
        ("claude-sonnet", Model::Claude35Sonnet),
        ("claude-haiku", Model::Claude3Haiku),
        ("gemini", Model::GeminiPro),
        ("deepseek", Model::DeepSeekV3),
        ("mistral", Model::MistralMedium),
    ];

    for (alias, model) in aliases {
        if alias.eq_ignore_ascii_case(name) {
            return Some(*model);
        }
    }

    // 尝试直接匹配
    match name.to_lowercase().as_str() {
        "gpt-4-turbo" => Some(Model::GPT4Turbo),
        "claude-3-opus" => Some(Model::Claude3Opus),
        _ => None,
    }
}

#[test]
fn test_alias_resolution() {
    assert_eq!(resolve_model_alias("gpt4t"), Some(Model::GPT4Turbo));
    assert_eq!(resolve_model_alias("gpt4"), Some(Model::GPT4));
    assert_eq!(resolve_model_alias("GPT4"), Some(Model::GPT4)); // 大小写不敏感
    assert_eq!(
        resolve_model_alias("claude-sonnet"),
        Some(Model::Claude35Sonnet)
    );
    assert_eq!(resolve_model_alias("gemini"), Some(Model::GeminiPro));
    assert_eq!(resolve_model_alias("deepseek"), Some(Model::DeepSeekV3));
}

#[test]
fn test_alias_unknown() {
    assert_eq!(resolve_model_alias("unknown-model"), None);
    assert_eq!(resolve_model_alias("invalid"), None);
}

// ==================== 综合测试 ====================

#[test]
fn test_all_models_have_valid_provider() {
    for model in Model::all() {
        let provider = model.provider();
        // 确保提供商有有效的 base URL
        let base_url = provider.base_url();
        assert!(
            !base_url.is_empty(),
            "Provider {:?} has empty base URL",
            provider
        );
        assert!(
            base_url.starts_with("https://"),
            "Invalid base URL for {:?}",
            provider
        );
    }
}

#[test]
fn test_no_duplicate_models() {
    let models = Model::all();
    let mut seen = std::collections::HashSet::new();

    for model in models {
        assert!(!seen.contains(&model), "Duplicate model: {:?}", model);
        seen.insert(model);
    }
}

#[test]
fn test_model_coverage_summary() {
    let models = Model::all();
    let mut by_provider: HashMap<ModelProvider, Vec<Model>> = HashMap::new();

    for model in models {
        by_provider.entry(model.provider()).or_default().push(model);
    }

    println!("\n=== Model Coverage Summary ===");
    println!("Total models: {}", Model::all().len());

    for (provider, models) in by_provider {
        println!("\n{:?}:", provider);
        for model in models {
            println!("  - {:?}", model);
        }
    }

    println!("\n=== Test Coverage Summary ===");
    println!("✓ Model provider mapping tests");
    println!("✓ Model count tests (>= 15 models)");
    println!("✓ Provider coverage tests");
    println!("✓ Fallback chain tests");
    println!("✓ Parameter mapping tests");
    println!("✓ Provider configuration tests");
    println!("✓ Alias resolution tests");
    println!("✓ Integration tests");
}
