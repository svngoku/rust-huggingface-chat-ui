use serde::{Deserialize, Serialize};
use std::error::Error;

/// Configuration for the chat client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub base_url: String,
    pub token: String,
    pub model: String,
    pub max_tokens: Option<i64>,
    pub temperature: Option<f64>,
    pub system_prompt: Option<String>,
    pub max_context_messages: usize,
}

impl ChatConfig {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        dotenv::dotenv().ok();

        let base_url = std::env::var("HF_BASE_URL").unwrap_or_else(|_| {
            eprintln!("\n⚠️  No HF_BASE_URL set. Please check WORKING_CONFIGS.md for setup instructions.");
            eprintln!("   Tip: For free local usage, install Ollama: brew install ollama && ollama pull llama3.2\n");
            "http://localhost:11434/v1".to_string()
        });

        let token = std::env::var("HUGGINGFACE_TOKEN").unwrap_or_else(|_| {
            if base_url.contains("localhost") || base_url.contains("127.0.0.1") {
                "unused".to_string()
            } else {
                eprintln!("\n⚠️  HUGGINGFACE_TOKEN not set but using remote API!");
                eprintln!("   Please set your API token in the .env file.\n");
                "missing-token".to_string()
            }
        });

        let model = std::env::var("HF_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
        let system_prompt = std::env::var("SYSTEM_PROMPT").ok();

        Ok(Self {
            base_url,
            token,
            model,
            max_tokens: Some(500),
            temperature: Some(0.7),
            system_prompt,
            max_context_messages: 20,
        })
    }
}
