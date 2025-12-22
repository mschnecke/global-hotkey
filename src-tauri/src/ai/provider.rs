//! AI Provider trait and common types

use crate::error::AppError;

/// Response from an AI provider
#[derive(Debug, Clone)]
pub struct AiResponse {
    pub text: String,
}

/// Trait for AI providers
pub trait AiProvider: Send + Sync {
    /// Send a text prompt to the AI
    fn send_text(
        &self,
        system_prompt: &str,
        user_input: &str,
    ) -> impl std::future::Future<Output = Result<AiResponse, AppError>> + Send;

    /// Send audio data to the AI
    fn send_audio(
        &self,
        system_prompt: &str,
        audio_data: &[u8],
        mime_type: &str,
    ) -> impl std::future::Future<Output = Result<AiResponse, AppError>> + Send;

    /// Test the connection/API key
    fn test_connection(&self) -> impl std::future::Future<Output = Result<bool, AppError>> + Send;
}
