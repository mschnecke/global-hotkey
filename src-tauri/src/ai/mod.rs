//! AI Module - Handles AI provider integrations

pub mod gemini;
pub mod provider;
pub mod roles;

pub use gemini::GeminiProvider;
pub use provider::AiProvider;
pub use roles::get_builtin_roles;
