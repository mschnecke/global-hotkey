//! Audio encoding to Opus (default) and WAV (fallback) formats

use crate::error::AppError;
use audiopus::{coder::Encoder, Application, Channels, SampleRate};
use std::io::Cursor;

/// Encode PCM samples to Opus in Ogg container (default, ~10x smaller than WAV)
pub fn encode_to_opus(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<u8>, AppError> {
    // Opus requires specific sample rates: 8000, 12000, 16000, 24000, 48000
    // Resample if needed
    let (resampled, target_rate) = resample_for_opus(samples, sample_rate);

    // Convert f32 to i16
    let samples_i16: Vec<i16> = resampled
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        })
        .collect();

    // Create Opus encoder
    let opus_channels = if channels == 1 {
        Channels::Mono
    } else {
        Channels::Stereo
    };

    let opus_sample_rate = match target_rate {
        8000 => SampleRate::Hz8000,
        12000 => SampleRate::Hz12000,
        16000 => SampleRate::Hz16000,
        24000 => SampleRate::Hz24000,
        _ => SampleRate::Hz48000,
    };

    let mut encoder = Encoder::new(opus_sample_rate, opus_channels, Application::Voip)
        .map_err(|e| AppError::Audio(format!("Failed to create Opus encoder: {}", e)))?;

    // Set bitrate for voice (lower = smaller file, still good quality for speech)
    encoder
        .set_bitrate(audiopus::Bitrate::BitsPerSecond(24000))
        .map_err(|e| AppError::Audio(format!("Failed to set bitrate: {}", e)))?;

    // Frame size: 20ms at target sample rate
    let frame_size = (target_rate as usize) / 50; // 20ms frames
    let mut opus_output = Vec::new();

    // Encode frames
    for chunk in samples_i16.chunks(frame_size * channels as usize) {
        if chunk.len() < frame_size * channels as usize {
            // Pad last frame if needed
            let mut padded = chunk.to_vec();
            padded.resize(frame_size * channels as usize, 0);

            let mut buffer = vec![0u8; 4000];
            let encoded_len = encoder
                .encode(&padded, &mut buffer)
                .map_err(|e| AppError::Audio(format!("Opus encoding failed: {}", e)))?;
            opus_output.extend_from_slice(&buffer[..encoded_len]);
        } else {
            let mut buffer = vec![0u8; 4000];
            let encoded_len = encoder
                .encode(chunk, &mut buffer)
                .map_err(|e| AppError::Audio(format!("Opus encoding failed: {}", e)))?;
            opus_output.extend_from_slice(&buffer[..encoded_len]);
        }
    }

    // Wrap in Ogg container
    let ogg_data = wrap_in_ogg(&opus_output, target_rate, channels)?;

    Ok(ogg_data)
}

/// Resample audio to a rate supported by Opus
fn resample_for_opus(samples: &[f32], sample_rate: u32) -> (Vec<f32>, u32) {
    // Opus supported rates: 8000, 12000, 16000, 24000, 48000
    let target_rate = match sample_rate {
        r if r <= 8000 => 8000,
        r if r <= 12000 => 12000,
        r if r <= 16000 => 16000,
        r if r <= 24000 => 24000,
        _ => 48000,
    };

    if sample_rate == target_rate {
        return (samples.to_vec(), target_rate);
    }

    // Simple linear resampling
    let ratio = target_rate as f64 / sample_rate as f64;
    let new_len = (samples.len() as f64 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;

        let sample = if src_idx + 1 < samples.len() {
            samples[src_idx] * (1.0 - frac as f32) + samples[src_idx + 1] * frac as f32
        } else if src_idx < samples.len() {
            samples[src_idx]
        } else {
            0.0
        };

        resampled.push(sample);
    }

    (resampled, target_rate)
}

/// Wrap Opus frames in an Ogg container
fn wrap_in_ogg(opus_data: &[u8], sample_rate: u32, channels: u16) -> Result<Vec<u8>, AppError> {
    use ogg::writing::PacketWriter;

    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);

    {
        let mut writer = PacketWriter::new(&mut cursor);

        // Write Opus identification header
        let mut id_header = Vec::new();
        id_header.extend_from_slice(b"OpusHead");
        id_header.push(1); // Version
        id_header.push(channels as u8); // Channel count
        id_header.extend_from_slice(&0u16.to_le_bytes()); // Pre-skip
        id_header.extend_from_slice(&sample_rate.to_le_bytes()); // Input sample rate
        id_header.extend_from_slice(&0i16.to_le_bytes()); // Output gain
        id_header.push(0); // Channel mapping family

        writer
            .write_packet(id_header, 0, ogg::writing::PacketWriteEndInfo::EndPage, 0)
            .map_err(|e| AppError::Audio(format!("Failed to write Opus header: {}", e)))?;

        // Write Opus comment header
        let mut comment_header = Vec::new();
        comment_header.extend_from_slice(b"OpusTags");
        let vendor = b"global-hotkey";
        comment_header.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        comment_header.extend_from_slice(vendor);
        comment_header.extend_from_slice(&0u32.to_le_bytes()); // No comments

        writer
            .write_packet(
                comment_header,
                0,
                ogg::writing::PacketWriteEndInfo::EndPage,
                0,
            )
            .map_err(|e| AppError::Audio(format!("Failed to write Opus comment: {}", e)))?;

        // Write audio data as a single packet
        writer
            .write_packet(
                opus_data.to_vec(),
                0,
                ogg::writing::PacketWriteEndInfo::EndStream,
                0,
            )
            .map_err(|e| AppError::Audio(format!("Failed to write Opus data: {}", e)))?;
    }

    Ok(output)
}

/// Encode PCM samples to WAV format (fallback)
pub fn encode_to_wav(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<u8>, AppError> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| AppError::Audio(format!("Failed to create WAV writer: {}", e)))?;

        for &sample in samples {
            // Clamp and convert f32 to i16
            let clamped = sample.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * 32767.0) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| AppError::Audio(format!("Failed to write sample: {}", e)))?;
        }

        writer
            .finalize()
            .map_err(|e| AppError::Audio(format!("Failed to finalize WAV: {}", e)))?;
    }

    Ok(cursor.into_inner())
}

/// Get the MIME type for Opus audio
pub fn opus_mime_type() -> &'static str {
    "audio/ogg"
}

/// Get the MIME type for WAV audio
pub fn wav_mime_type() -> &'static str {
    "audio/wav"
}
