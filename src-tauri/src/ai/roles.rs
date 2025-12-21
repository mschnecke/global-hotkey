//! Built-in AI roles

use crate::config::schema::{AiRole, OutputFormat};

/// Get the default built-in roles
pub fn get_builtin_roles() -> Vec<AiRole> {
    vec![
        AiRole {
            id: "de-transcribe".to_string(),
            name: "DE Transcribe".to_string(),
            system_prompt: "Transcribe the following German audio accurately. Output only the transcription without any additional commentary.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
        AiRole {
            id: "de-en-translate".to_string(),
            name: "DE→EN Translate".to_string(),
            system_prompt: "Translate the following German text to English. Maintain the original meaning and tone.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
        AiRole {
            id: "beautify".to_string(),
            name: "Beautify Text".to_string(),
            system_prompt: "Improve the formatting, grammar, and clarity of this text while preserving its meaning.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
        AiRole {
            id: "ai-response".to_string(),
            name: "Format as AI Response".to_string(),
            system_prompt: "Format this text as a professional, well-structured response suitable for an AI assistant.".to_string(),
            output_format: OutputFormat::Plain,
            is_builtin: true,
        },
    ]
}
