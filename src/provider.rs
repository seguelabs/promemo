use crate::config::{Config, ProviderConfig};
use crate::models::{memory_input_schema, MemoryInput};
use anyhow::{bail, Context};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

pub trait MemoryProvider {
    fn extract_memory(&self, request: &ExtractionRequest) -> anyhow::Result<MemoryInput>;
}

#[derive(Debug, Clone)]
pub struct ExtractionRequest {
    pub feature: String,
    pub source_text: String,
}

pub fn provider_from_config(config: &Config) -> anyhow::Result<Box<dyn MemoryProvider>> {
    match config.provider.kind.as_str() {
        "openai-compatible" => Ok(Box::new(OpenAiCompatibleProvider::from_config(
            &config.provider,
        )?)),
        other => bail!("unsupported provider kind `{}`", other),
    }
}

pub struct OpenAiCompatibleProvider {
    client: Client,
    config: ProviderConfig,
    api_key: String,
}

impl OpenAiCompatibleProvider {
    pub fn from_config(config: &ProviderConfig) -> anyhow::Result<Self> {
        let api_key = std::env::var(&config.api_key_env).with_context(|| {
            format!(
                "provider API key env var `{}` is not set",
                config.api_key_env
            )
        })?;
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;
        Ok(Self {
            client,
            config: config.clone(),
            api_key,
        })
    }

    fn chat_completions_url(&self) -> String {
        format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }
}

impl MemoryProvider for OpenAiCompatibleProvider {
    fn extract_memory(&self, request: &ExtractionRequest) -> anyhow::Result<MemoryInput> {
        let body = json!({
            "model": self.config.model,
            "temperature": 0,
            "response_format": { "type": "json_object" },
            "messages": [
                {
                    "role": "system",
                    "content": extraction_system_prompt()
                },
                {
                    "role": "user",
                    "content": format!(
                        "Feature slug or name: {}\n\nPromemo MemoryInput JSON schema:\n{}\n\nUnstructured notes:\n{}",
                        request.feature,
                        serde_json::to_string_pretty(&memory_input_schema())?,
                        request.source_text
                    )
                }
            ]
        });

        let response = self
            .client
            .post(self.chat_completions_url())
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .context("provider request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            bail!("provider request failed with {}: {}", status, text);
        }

        let completion: ChatCompletionResponse = response
            .json()
            .context("provider returned invalid JSON response")?;
        parse_memory_from_chat_completion(completion)
    }
}

fn extraction_system_prompt() -> &'static str {
    "You extract durable project memory for Promemo. Return only valid JSON matching the supplied MemoryInput schema. Do not include raw transcript text. Summarize decisions, files, APIs, data models, todos, open questions, future work, and useful prompts when they are present. Use todo priority low, normal, or high."
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    content: String,
}

fn parse_memory_from_chat_completion(
    completion: ChatCompletionResponse,
) -> anyhow::Result<MemoryInput> {
    let content = completion
        .choices
        .first()
        .map(|choice| choice.message.content.trim())
        .filter(|content| !content.is_empty())
        .context("provider response did not include message content")?;
    let memory: MemoryInput =
        serde_json::from_str(content).context("provider output was not valid MemoryInput JSON")?;
    memory.validate()?;
    Ok(memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_memory_from_chat_completion_content() {
        let completion = ChatCompletionResponse {
            choices: vec![ChatChoice {
                message: ChatMessage {
                    content: serde_json::to_string(&MemoryInput {
                        title: "Authentication".to_string(),
                        summary: "JWT authentication memory.".to_string(),
                        ..MemoryInput::default()
                    })
                    .unwrap(),
                },
            }],
        };

        let memory = parse_memory_from_chat_completion(completion).unwrap();

        assert_eq!(memory.title, "Authentication");
        assert_eq!(memory.summary, "JWT authentication memory.");
    }
}
