//! Gemini API client implementation

use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{AiProvider, AiResponse};
use crate::error::AppError;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_MODEL: &str = "gemini-2.5-flash-lite";

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/models/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        )
    }
}

// Request/Response types
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

impl AiProvider for GeminiProvider {
    async fn send_text(
        &self,
        system_prompt: &str,
        user_input: &str,
    ) -> Result<AiResponse, AppError> {
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_input);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part::Text {
                    text: combined_prompt,
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.1,
                max_output_tokens: 8192,
            }),
        };

        let response = self
            .client
            .post(&self.endpoint())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = gemini_response.error {
            return Err(AppError::Ai(error.message));
        }

        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| p.text)
            .unwrap_or_default();

        Ok(AiResponse {
            text,
            finish_reason: None,
        })
    }

    async fn send_audio(
        &self,
        system_prompt: &str,
        audio_data: &[u8],
        mime_type: &str,
    ) -> Result<AiResponse, AppError> {
        let audio_base64 = base64::engine::general_purpose::STANDARD.encode(audio_data);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text {
                        text: system_prompt.to_string(),
                    },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: audio_base64,
                        },
                    },
                ],
            }],
            generation_config: Some(GenerationConfig {
                temperature: 0.1,
                max_output_tokens: 8192,
            }),
        };

        let response = self
            .client
            .post(&self.endpoint())
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = gemini_response.error {
            return Err(AppError::Ai(error.message));
        }

        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .and_then(|p| p.text)
            .unwrap_or_default();

        Ok(AiResponse {
            text,
            finish_reason: None,
        })
    }

    async fn test_connection(&self) -> Result<bool, AppError> {
        let result = self.send_text("Respond with only: OK", "Test").await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }
}
