use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Frames per waveform peak.
pub const PEAK_BLOCK: usize = 512;
/// Safety cap so a stray multi-hour file cannot eat all RAM (stereo f32).
const MAX_SECONDS: f64 = 3.0 * 3600.0;

/// Decoded audio, interleaved stereo f32.
pub struct Pcm {
    pub rate: u32,
    pub samples: Vec<f32>,
    pub peaks: Vec<f32>,
}

impl Pcm {
    pub fn frames(&self) -> usize {
        self.samples.len() / 2
    }

    pub fn seconds(&self) -> f64 {
        self.frames() as f64 / self.rate.max(1) as f64
    }

    /// Stereo sample pair at a (fractional) frame index, zero outside the file.
    pub fn at(&self, pos: f64) -> (f32, f32) {
        if pos < 0.0 || !pos.is_finite() {
            return (0.0, 0.0);
        }
        let i = pos.floor() as usize;
        let frac = (pos - i as f64) as f32;
        let get = |k: usize| -> (f32, f32) {
            match (self.samples.get(k * 2), self.samples.get(k * 2 + 1)) {
                (Some(l), Some(r)) => (*l, *r),
                _ => (0.0, 0.0),
            }
        };
        if i >= self.frames() {
            return (0.0, 0.0);
        }
        let (l0, r0) = get(i);
        let (l1, r1) = get(i + 1);
        (l0 + (l1 - l0) * frac, r0 + (r1 - r0) * frac)
    }
}

fn open(path: &Path) -> Result<(Box<dyn symphonia::core::formats::FormatReader>, u32), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Format nicht erkannt: {e}"))?;
    let format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| {
            t.codec_params.codec != CODEC_TYPE_NULL
                && t.codec_params.sample_rate.is_some()
                && t.codec_params.channels.is_some()
        })
        .ok_or("Keine Tonspur gefunden")?;
    let id = track.id;
    Ok((format, id))
}

/// Duration in seconds from the container header, without decoding.
pub fn probe(path: &Path) -> Result<f64, String> {
    let (format, id) = open(path)?;
    let params = &format
        .tracks()
        .iter()
        .find(|t| t.id == id)
        .ok_or("Keine Tonspur gefunden")?
        .codec_params;
    match (params.n_frames, params.sample_rate) {
        (Some(n), Some(rate)) if rate > 0 => Ok(n as f64 / rate as f64),
        _ => decode_file(path).map(|p| p.seconds()),
    }
}

pub fn decode_file(path: &Path) -> Result<Pcm, String> {
    let (mut format, track_id) = open(path)?;
    let params = format
        .tracks()
        .iter()
        .find(|t| t.id == track_id)
        .ok_or("Keine Tonspur gefunden")?
        .codec_params
        .clone();
    let rate = params.sample_rate.ok_or("Unbekannte Abtastrate")?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&params, &DecoderOptions::default())
        .map_err(|e| format!("Codec nicht unterstützt: {e}"))?;
    let max_samples = (MAX_SECONDS * rate as f64) as usize * 2;
    let mut samples: Vec<f32> = Vec::new();
    let mut buf: Option<SampleBuffer<f32>> = None;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(Error::ResetRequired) => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(Error::DecodeError(_)) => continue, // one bad packet is a glitch, not a failure
            Err(Error::IoError(_)) => break,
            Err(e) => return Err(e.to_string()),
        };
        let spec = *decoded.spec();
        let channels = spec.channels.count().max(1);
        let b =
            buf.get_or_insert_with(|| SampleBuffer::<f32>::new(decoded.capacity() as u64, spec));
        if (b.capacity() as u64) < decoded.capacity() as u64 * channels as u64 {
            *b = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        }
        b.copy_interleaved_ref(decoded);
        for frame in b.samples().chunks_exact(channels) {
            let (l, r) = match frame {
                [m] => (*m, *m),
                [l, r, ..] => (*l, *r),
                [] => (0.0, 0.0),
            };
            samples.push(l);
            samples.push(r);
        }
        if samples.len() >= max_samples {
            break;
        }
    }
    if samples.is_empty() {
        return Err("Die Datei enthält keinen hörbaren Ton".into());
    }
    let peaks = samples
        .chunks(PEAK_BLOCK * 2)
        .map(|c| c.iter().fold(0.0f32, |m, s| m.max(s.abs())))
        .collect();
    Ok(Pcm {
        rate,
        samples,
        peaks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(frames: usize, rate: u32) -> Vec<u8> {
        let data_len = (frames * 2) as u32;
        let mut v = Vec::new();
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&(36 + data_len).to_le_bytes());
        v.extend_from_slice(b"WAVEfmt ");
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&rate.to_le_bytes());
        v.extend_from_slice(&(rate * 2).to_le_bytes());
        v.extend_from_slice(&2u16.to_le_bytes());
        v.extend_from_slice(&16u16.to_le_bytes());
        v.extend_from_slice(b"data");
        v.extend_from_slice(&data_len.to_le_bytes());
        for i in 0..frames {
            let s = ((i as f32 * 0.05).sin() * 16000.0) as i16;
            v.extend_from_slice(&s.to_le_bytes());
        }
        v
    }

    #[test]
    fn decodes_wav_to_stereo_and_reports_duration() {
        let dir = std::env::temp_dir().join(format!("hk-audio-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.wav");
        std::fs::write(&path, wav(44100, 44100)).unwrap();
        let pcm = decode_file(&path).unwrap();
        assert_eq!(pcm.rate, 44100);
        assert_eq!(pcm.frames(), 44100);
        assert!((probe(&path).unwrap() - 1.0).abs() < 0.01);
        assert!(pcm.peaks.iter().any(|p| *p > 0.4));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn garbage_file_is_an_error() {
        let dir = std::env::temp_dir().join(format!("hk-audio-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("x.mp3");
        std::fs::write(&path, b"not audio at all, just some text").unwrap();
        assert!(decode_file(&path).is_err());
        assert!(decode_file(&dir.join("missing.wav")).is_err());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn at_interpolates_and_is_silent_outside() {
        let pcm = Pcm {
            rate: 10,
            samples: vec![0.0, 0.0, 1.0, 1.0],
            peaks: vec![],
        };
        assert_eq!(pcm.at(0.5), (0.5, 0.5));
        assert_eq!(pcm.at(-1.0), (0.0, 0.0));
        assert_eq!(pcm.at(5.0), (0.0, 0.0));
        assert_eq!(pcm.at(f64::NAN), (0.0, 0.0));
    }
}
