//! MP4 export through AVAssetWriter: hardware H.264 / HEVC on Apple Silicon and
//! AAC audio. Frames arrive as RGBA8, audio as interleaved stereo f32.

#![allow(deprecated, clippy::chunks_exact_to_as_chunks)] // the objc2 CoreVideo/CoreMedia free functions still work; their method forms are not wired for every type yet

use crate::MediaError;
use block2::RcBlock;
use hk_core::Fps;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::*;
use objc2_core_audio_types::{
    AudioStreamBasicDescription, kAudioFormatFlagIsFloat, kAudioFormatFlagIsPacked,
    kAudioFormatLinearPCM, kAudioFormatMPEG4AAC,
};
use objc2_core_foundation::CFRetained;
use objc2_core_media::*;
use objc2_core_video::*;
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
use std::ffi::c_void;
use std::path::Path;
use std::ptr::NonNull;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Runs `f`; an Objective-C exception raised inside becomes a normal error.
fn guard<R>(f: impl FnOnce() -> Result<R, MediaError>) -> Result<R, MediaError> {
    match objc2::exception::catch(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(Some(e)) => Err(err(format!(
            "AVFoundation lehnt die Einstellungen ab: {e:?}"
        ))),
        Err(None) => Err(err("AVFoundation lehnt die Einstellungen ab")),
    }
}

fn err(msg: impl Into<String>) -> MediaError {
    MediaError::Decode(msg.into())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Codec {
    H264,
    Hevc,
}

pub struct VideoWriter {
    writer: Retained<AVAssetWriter>,
    video: Retained<AVAssetWriterInput>,
    adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
    audio: Option<Retained<AVAssetWriterInput>>,
    width: usize,
    height: usize,
    fps: Fps,
    audio_rate: u32,
    audio_frames_written: u64,
    audio_finished: bool,
    video_finished: bool,
}

fn dict(
    pairs: Vec<(&NSString, Retained<AnyObject>)>,
) -> Retained<NSDictionary<NSString, AnyObject>> {
    let keys: Vec<&NSString> = pairs.iter().map(|(k, _)| *k).collect();
    let values: Vec<Retained<AnyObject>> = pairs.into_iter().map(|(_, v)| v).collect();
    NSDictionary::from_retained_objects(&keys, &values)
}

fn num_u(v: usize) -> Retained<AnyObject> {
    Retained::into_super(Retained::into_super(NSNumber::numberWithUnsignedInteger(v))).into()
}

fn num_f(v: f64) -> Retained<AnyObject> {
    Retained::into_super(Retained::into_super(NSNumber::numberWithDouble(v))).into()
}

fn as_any<T: objc2::Message>(v: Retained<T>) -> Retained<AnyObject> {
    // SAFETY: every Objective-C object is an `AnyObject`.
    unsafe { Retained::cast_unchecked(v) }
}

/// What to write: picture size, frame rate, video bit rate and codec, and optionally an AAC track.
#[derive(Clone, Copy, Debug)]
pub struct WriterConfig {
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    /// Video bit rate in bit/s.
    pub bitrate: u32,
    pub codec: Codec,
    /// `Some((sample rate in Hz, bit rate in bit/s))` adds an AAC track.
    pub audio: Option<(u32, u32)>,
}

impl VideoWriter {
    /// Opens the file for writing. AVFoundation answers invalid settings with an Objective-C
    /// exception; every call below turns that into an error instead of letting it end the app.
    pub fn create(path: &Path, config: &WriterConfig) -> Result<VideoWriter, MediaError> {
        guard(|| Self::create_inner(path, config))
    }

    fn create_inner(path: &Path, config: &WriterConfig) -> Result<VideoWriter, MediaError> {
        let WriterConfig {
            width,
            height,
            fps,
            bitrate,
            codec,
            audio,
        } = *config;
        let (audio_rate, audio_bitrate) = match audio {
            Some((rate, bits)) => (Some(rate), bits),
            None => (None, 192_000),
        };
        let (w, h) = (
            (width.max(16) & !1) as usize,
            (height.max(16) & !1) as usize,
        );
        let _ = std::fs::remove_file(path); // AVAssetWriter refuses to overwrite
        let path_str = path
            .to_str()
            .ok_or_else(|| err("Pfad ist kein gültiger Text"))?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path_str));
        // SAFETY: all calls below pass valid objects created in this function.
        unsafe {
            let file_type = AVFileTypeMPEG4.ok_or_else(|| err("AVFoundation fehlt"))?;
            let writer = AVAssetWriter::assetWriterWithURL_fileType_error(&url, file_type)
                .map_err(|e| err(e.localizedDescription().to_string()))?;

            let key =
                |k: Option<&'static NSString>| k.ok_or_else(|| err("AVFoundation-Konstante fehlt"));
            let codec_type = match codec {
                Codec::H264 => AVVideoCodecTypeH264,
                Codec::Hevc => AVVideoCodecTypeHEVC,
            }
            .ok_or_else(|| err("Codec nicht verfügbar"))?;
            let compression = dict(vec![(
                key(AVVideoAverageBitRateKey)?,
                num_u(bitrate as usize),
            )]);
            let settings = dict(vec![
                (
                    key(AVVideoCodecKey)?,
                    as_any(Retained::<NSString>::from(codec_type)),
                ),
                (key(AVVideoWidthKey)?, num_u(w)),
                (key(AVVideoHeightKey)?, num_u(h)),
                (key(AVVideoCompressionPropertiesKey)?, as_any(compression)),
            ]);
            let media_video = AVMediaTypeVideo.ok_or_else(|| err("AVFoundation fehlt"))?;
            let video = AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(
                media_video,
                Some(&settings),
            );
            video.setExpectsMediaDataInRealTime(false);

            let pixel_key = NSString::from_str("PixelFormatType");
            let width_key = NSString::from_str("Width");
            let height_key = NSString::from_str("Height");
            let attrs = dict(vec![
                (&pixel_key, num_u(kCVPixelFormatType_32BGRA as usize)),
                (&width_key, num_u(w)),
                (&height_key, num_u(h)),
            ]);
            let adaptor = AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(&video, Some(&attrs));
            if !writer.canAddInput(&video) {
                return Err(err("Video-Eingang nicht möglich"));
            }
            writer.addInput(&video);

            let audio = match audio_rate {
                Some(rate) => {
                    let format_key = NSString::from_str("AVFormatIDKey");
                    let ch_key = NSString::from_str("AVNumberOfChannelsKey");
                    let rate_key = NSString::from_str("AVSampleRateKey");
                    let br_key = NSString::from_str("AVEncoderBitRateKey");
                    let a_settings = dict(vec![
                        (&format_key, num_u(kAudioFormatMPEG4AAC as usize)),
                        (&ch_key, num_u(2)),
                        (&rate_key, num_f(rate as f64)),
                        (
                            &br_key,
                            num_u(audio_bitrate.clamp(32_000, 512_000) as usize),
                        ),
                    ]);
                    let media_audio = AVMediaTypeAudio.ok_or_else(|| err("AVFoundation fehlt"))?;
                    let input = AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(
                        media_audio,
                        Some(&a_settings),
                    );
                    input.setExpectsMediaDataInRealTime(false);
                    if !writer.canAddInput(&input) {
                        return Err(err("Audio-Eingang nicht möglich"));
                    }
                    writer.addInput(&input);
                    Some(input)
                }
                None => None,
            };
            if !writer.startWriting() {
                let why = writer
                    .error()
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_default();
                return Err(err(format!("Schreiben konnte nicht starten: {why}")));
            }
            writer.startSessionAtSourceTime(CMTime::new(0, 600));
            Ok(VideoWriter {
                writer,
                video,
                adaptor,
                audio,
                width: w,
                height: h,
                fps,
                audio_rate: audio_rate.unwrap_or(48_000),
                audio_frames_written: 0,
                audio_finished: false,
                video_finished: false,
            })
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    /// Fails once AVFoundation has put the writer into its failed state.
    pub fn check(&self) -> Result<(), MediaError> {
        // SAFETY: property reads on live objects.
        unsafe {
            if self.writer.status() == AVAssetWriterStatus::Failed {
                let why = self
                    .writer
                    .error()
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_default();
                return Err(err(format!("Export fehlgeschlagen: {why}")));
            }
        }
        Ok(())
    }

    pub fn video_ready(&self) -> bool {
        // SAFETY: property read.
        unsafe { self.video.isReadyForMoreMediaData() }
    }

    pub fn audio_ready(&self) -> bool {
        // SAFETY: property read.
        self.audio
            .as_ref()
            .is_some_and(|a| unsafe { a.isReadyForMoreMediaData() })
    }

    pub fn has_audio(&self) -> bool {
        self.audio.is_some()
    }

    /// No more sound will follow. The writer waits for this before it accepts picture that runs
    /// ahead of the last audio sample, so call it as soon as the sound is complete.
    pub fn finish_audio(&mut self) {
        if let (Some(a), false) = (&self.audio, self.audio_finished) {
            // SAFETY: valid input, finished once.
            let _ = guard(|| {
                unsafe { a.markAsFinished() };
                Ok(())
            });
            self.audio_finished = true;
        }
    }

    /// No more picture will follow (see [`VideoWriter::finish_audio`]).
    pub fn finish_video(&mut self) {
        if !self.video_finished {
            // SAFETY: valid input, finished once.
            let _ = guard(|| {
                unsafe { self.video.markAsFinished() };
                Ok(())
            });
            self.video_finished = true;
        }
    }

    /// Append frame number `index`. `rgba` must be `width * height * 4` bytes (the size from
    /// [`VideoWriter::size`]). Only call when [`VideoWriter::video_ready`] is true.
    pub fn push_video(&mut self, rgba: &[u8], index: i64) -> Result<(), MediaError> {
        guard(|| self.push_video_inner(rgba, index))
    }

    fn push_video_inner(&mut self, rgba: &[u8], index: i64) -> Result<(), MediaError> {
        if rgba.len() < self.width * self.height * 4 {
            return Err(err("Bild zu klein für den Export"));
        }
        // SAFETY: the pixel buffer is locked while we write into it and each row stays within
        // `bytes_per_row`, which CoreVideo guarantees is at least `width * 4`.
        unsafe {
            let pool = self
                .adaptor
                .pixelBufferPool()
                .ok_or_else(|| err("Kein Pixelpuffer-Pool"))?;
            let mut raw: *mut CVPixelBuffer = std::ptr::null_mut();
            let status =
                CVPixelBufferPool::create_pixel_buffer(None, &pool, NonNull::from(&mut raw));
            let raw = NonNull::new(raw)
                .filter(|_| status == 0)
                .ok_or_else(|| err("Pixelpuffer konnte nicht angelegt werden"))?;
            let buffer: CFRetained<CVPixelBuffer> = CFRetained::from_raw(raw);
            CVPixelBufferLockBaseAddress(&buffer, CVPixelBufferLockFlags(0));
            let base = CVPixelBufferGetBaseAddress(&buffer) as *mut u8;
            let stride = CVPixelBufferGetBytesPerRow(&buffer);
            if base.is_null() || stride < self.width * 4 {
                CVPixelBufferUnlockBaseAddress(&buffer, CVPixelBufferLockFlags(0));
                return Err(err("Pixelpuffer nicht beschreibbar"));
            }
            for y in 0..self.height {
                let src = &rgba[y * self.width * 4..(y + 1) * self.width * 4];
                let dst = std::slice::from_raw_parts_mut(base.add(y * stride), self.width * 4);
                for (d, s) in dst
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .zip(src.chunks_exact(4))
                {
                    d[0] = s[2];
                    d[1] = s[1];
                    d[2] = s[0];
                    d[3] = 255;
                }
            }
            CVPixelBufferUnlockBaseAddress(&buffer, CVPixelBufferLockFlags(0));
            let time = CMTime::new(
                index.saturating_mul(self.fps.den as i64),
                self.fps.num as i32,
            );
            if !self
                .adaptor
                .appendPixelBuffer_withPresentationTime(&buffer, time)
            {
                let why = self
                    .writer
                    .error()
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_default();
                return Err(err(format!("Bild konnte nicht geschrieben werden: {why}")));
            }
        }
        Ok(())
    }

    /// Append interleaved stereo f32 samples that directly follow the previous call.
    /// Only call when [`VideoWriter::audio_ready`] is true.
    pub fn push_audio(&mut self, stereo: &[f32]) -> Result<(), MediaError> {
        guard(|| self.push_audio_inner(stereo))
    }

    fn push_audio_inner(&mut self, stereo: &[f32]) -> Result<(), MediaError> {
        let Some(input) = self.audio.clone() else {
            return Ok(());
        };
        let frames = stereo.len() / 2;
        if frames == 0 {
            return Ok(());
        }
        let bytes = frames * 8;
        // SAFETY: the block buffer owns a fresh allocation of `bytes` bytes into which we copy
        // exactly `bytes` bytes; all out-pointers point at live locals.
        unsafe {
            let asbd = AudioStreamBasicDescription {
                mSampleRate: self.audio_rate as f64,
                mFormatID: kAudioFormatLinearPCM,
                mFormatFlags: kAudioFormatFlagIsFloat | kAudioFormatFlagIsPacked,
                mBytesPerPacket: 8,
                mFramesPerPacket: 1,
                mBytesPerFrame: 8,
                mChannelsPerFrame: 2,
                mBitsPerChannel: 32,
                mReserved: 0,
            };
            let mut desc: *const CMAudioFormatDescription = std::ptr::null();
            let status = CMAudioFormatDescriptionCreate(
                None,
                NonNull::from(&asbd),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                None,
                NonNull::from(&mut desc),
            );
            let desc = desc
                .as_ref()
                .filter(|_| status == 0)
                .ok_or_else(|| err("Audioformat nicht verfügbar"))?;

            let mut block: *mut CMBlockBuffer = std::ptr::null_mut();
            let status = CMBlockBufferCreateWithMemoryBlock(
                None,
                std::ptr::null_mut(),
                bytes,
                None,
                std::ptr::null(),
                0,
                bytes,
                1 << 1,
                NonNull::from(&mut block),
            );
            let block = NonNull::new(block)
                .filter(|_| status == 0)
                .ok_or_else(|| err("Audiopuffer konnte nicht angelegt werden"))?;
            let block: CFRetained<CMBlockBuffer> = CFRetained::from_raw(block);
            let status = CMBlockBufferReplaceDataBytes(
                NonNull::new_unchecked(stereo.as_ptr() as *mut c_void),
                &block,
                0,
                bytes,
            );
            if status != 0 {
                return Err(err("Audiodaten konnten nicht kopiert werden"));
            }
            let pts = CMTime::new(self.audio_frames_written as i64, self.audio_rate as i32);
            let mut sample: *mut CMSampleBuffer = std::ptr::null_mut();
            let status = CMAudioSampleBufferCreateReadyWithPacketDescriptions(
                None,
                &block,
                desc,
                frames as isize,
                pts,
                std::ptr::null(),
                NonNull::from(&mut sample),
            );
            let sample = NonNull::new(sample)
                .filter(|_| status == 0)
                .ok_or_else(|| err("Audio-Sample konnte nicht angelegt werden"))?;
            let sample: CFRetained<CMSampleBuffer> = CFRetained::from_raw(sample);
            if !input.appendSampleBuffer(&sample) {
                let why = self
                    .writer
                    .error()
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_default();
                return Err(err(format!("Ton konnte nicht geschrieben werden: {why}")));
            }
        }
        self.audio_frames_written += frames as u64;
        Ok(())
    }

    /// Close the file. Blocks until AVFoundation has flushed everything.
    pub fn finish(self) -> Result<(), MediaError> {
        guard(|| self.finish_inner())
    }

    fn finish_inner(mut self) -> Result<(), MediaError> {
        self.finish_video();
        self.finish_audio();
        let (tx, rx) = channel();
        // SAFETY: inputs are marked finished before the writer is told to finish; the
        // completion block only sends on a channel.
        unsafe {
            let block = RcBlock::new(move || {
                let _ = tx.send(());
            });
            self.writer.finishWritingWithCompletionHandler(&block);
        }
        rx.recv_timeout(Duration::from_secs(120))
            .map_err(|_| err("Der Export wurde nicht rechtzeitig abgeschlossen"))?;
        // SAFETY: property reads.
        unsafe {
            if self.writer.status() != AVAssetWriterStatus::Completed {
                let why = self
                    .writer
                    .error()
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_default();
                return Err(err(format!("Export fehlgeschlagen: {why}")));
            }
        }
        Ok(())
    }

    /// Abort and remove the half-written file.
    pub fn cancel(self, path: &Path) {
        // SAFETY: cancelWriting is valid in any writing state.
        let _ = guard(|| {
            unsafe { self.writer.cancelWriting() };
            Ok(())
        });
        let _ = std::fs::remove_file(path);
    }
}
