//! Audio recording module

pub mod encoder;
pub mod recorder;

pub use encoder::{encode_to_opus, encode_to_wav, opus_mime_type};
pub use recorder::AudioRecorderHandle;
