//! Thin owners around the NDK media objects (MediaExtractor, MediaCodec, MediaMuxer, MediaFormat).
//! All `unsafe` of the Android media backend lives in this file; `decode.rs` and `encode.rs`
//! only use the safe API below.

use ndk_sys::*;
use std::ffi::{CStr, CString, c_char};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::Path;

pub const FLAG_EOS: u32 = AMEDIACODEC_BUFFER_FLAG_END_OF_STREAM as u32;
pub const FLAG_CONFIG: u32 = AMEDIACODEC_BUFFER_FLAG_CODEC_CONFIG as u32;

fn c(s: &str) -> CString {
    CString::new(s).unwrap_or_default()
}

fn ok(status: media_status_t, what: &str) -> Result<(), String> {
    if status == media_status_t::AMEDIA_OK {
        Ok(())
    } else {
        Err(format!("{what} (Fehler {})", status.0))
    }
}

// ---------------------------------------------------------------- MediaFormat

pub struct Format(*mut AMediaFormat);

impl Format {
    pub fn new() -> Format {
        // SAFETY: plain allocation; a null result is handled by every method below.
        Format(unsafe { AMediaFormat_new() })
    }

    fn adopt(ptr: *mut AMediaFormat) -> Option<Format> {
        (!ptr.is_null()).then_some(Format(ptr))
    }

    pub fn raw(&self) -> *const AMediaFormat {
        self.0
    }

    pub fn set_str(&mut self, key: &str, value: &str) {
        let (k, v) = (c(key), c(value));
        // SAFETY: valid format and NUL-terminated strings that outlive the call.
        unsafe { AMediaFormat_setString(self.0, k.as_ptr(), v.as_ptr()) }
    }

    pub fn set_i32(&mut self, key: &str, value: i32) {
        let k = c(key);
        // SAFETY: valid format and NUL-terminated key.
        unsafe { AMediaFormat_setInt32(self.0, k.as_ptr(), value) }
    }

    pub fn i32(&self, key: &str) -> Option<i32> {
        let k = c(key);
        let mut out = 0i32;
        // SAFETY: valid format, key and out pointer.
        unsafe { AMediaFormat_getInt32(self.0, k.as_ptr(), &mut out) }.then_some(out)
    }

    pub fn i64(&self, key: &str) -> Option<i64> {
        let k = c(key);
        let mut out = 0i64;
        // SAFETY: valid format, key and out pointer.
        unsafe { AMediaFormat_getInt64(self.0, k.as_ptr(), &mut out) }.then_some(out)
    }

    pub fn string(&self, key: &str) -> Option<String> {
        let k = c(key);
        let mut out: *const c_char = std::ptr::null();
        // SAFETY: valid format, key and out pointer; the returned string is owned by the format
        // and copied before the format can change.
        let found = unsafe { AMediaFormat_getString(self.0, k.as_ptr(), &mut out) };
        if !found || out.is_null() {
            return None;
        }
        // SAFETY: `out` is a NUL-terminated string valid while the format lives.
        Some(unsafe { CStr::from_ptr(out) }.to_string_lossy().into_owned())
    }
}

impl Drop for Format {
    fn drop(&mut self) {
        // SAFETY: owned, deleted once.
        unsafe { AMediaFormat_delete(self.0) };
    }
}

// ------------------------------------------------------------- MediaExtractor

pub struct Extractor {
    ptr: *mut AMediaExtractor,
    // Opened by us and passed as a descriptor: path based opening is refused on some devices.
    _file: File,
}

// SAFETY: the extractor is only ever used by the thread that currently owns the value.
unsafe impl Send for Extractor {}

impl Extractor {
    pub fn open(path: &Path) -> Result<Extractor, String> {
        let file = File::open(path).map_err(|e| format!("Die Datei lässt sich nicht öffnen: {e}"))?;
        let len = file.metadata().map_err(|e| e.to_string())?.len();
        // SAFETY: plain allocation.
        let ptr = unsafe { AMediaExtractor_new() };
        if ptr.is_null() {
            return Err("Der Medienleser lässt sich nicht anlegen".into());
        }
        let ex = Extractor { ptr, _file: file };
        // SAFETY: valid extractor; the descriptor stays open as long as `ex` lives.
        let status = unsafe {
            AMediaExtractor_setDataSourceFd(ex.ptr, ex._file.as_raw_fd(), 0, len as _)
        };
        ok(status, "Das Format wird nicht unterstützt")?;
        Ok(ex)
    }

    pub fn track_count(&self) -> usize {
        // SAFETY: valid extractor.
        unsafe { AMediaExtractor_getTrackCount(self.ptr) }
    }

    pub fn track_format(&self, i: usize) -> Option<Format> {
        // SAFETY: valid extractor; an out-of-range index yields null.
        Format::adopt(unsafe { AMediaExtractor_getTrackFormat(self.ptr, i) })
    }

    pub fn select(&mut self, i: usize) -> Result<(), String> {
        // SAFETY: valid extractor.
        ok(unsafe { AMediaExtractor_selectTrack(self.ptr, i) }, "Spur nicht wählbar")
    }

    pub fn seek(&mut self, us: i64) -> Result<(), String> {
        // SAFETY: valid extractor.
        let status = unsafe {
            AMediaExtractor_seekTo(self.ptr, us.max(0), SeekMode::AMEDIAEXTRACTOR_SEEK_PREVIOUS_SYNC)
        };
        ok(status, "Suchen fehlgeschlagen")
    }

    /// Reads the next sample of the selected track into `buf`: `(size, presentation time in µs)`.
    pub fn read(&mut self, buf: &mut [u8]) -> Option<(usize, i64)> {
        // SAFETY: valid extractor; `buf` is writable for `buf.len()` bytes.
        let n = unsafe { AMediaExtractor_readSampleData(self.ptr, buf.as_mut_ptr(), buf.len()) };
        if n < 0 {
            return None;
        }
        // SAFETY: valid extractor.
        let pts = unsafe { AMediaExtractor_getSampleTime(self.ptr) };
        Some((n as usize, pts.max(0)))
    }

    pub fn advance(&mut self) -> bool {
        // SAFETY: valid extractor.
        unsafe { AMediaExtractor_advance(self.ptr) }
    }
}

impl Drop for Extractor {
    fn drop(&mut self) {
        // SAFETY: owned, deleted once.
        unsafe { AMediaExtractor_delete(self.ptr) };
    }
}

// ----------------------------------------------------------------- MediaCodec

pub struct Codec {
    ptr: *mut AMediaCodec,
    started: bool,
}

// SAFETY: the codec is only ever used by the thread that currently owns the value.
unsafe impl Send for Codec {}

pub struct OutBuffer {
    pub index: usize,
    pub size: usize,
    pub pts: i64,
    pub flags: u32,
}

pub enum Output {
    Buffer(OutBuffer),
    FormatChanged,
    TryAgain,
}

impl Codec {
    fn adopt(ptr: *mut AMediaCodec, what: &str) -> Result<Codec, String> {
        if ptr.is_null() {
            Err(format!("Kein Codec für {what} auf diesem Gerät"))
        } else {
            Ok(Codec {
                ptr,
                started: false,
            })
        }
    }

    pub fn decoder(mime: &str) -> Result<Codec, String> {
        let m = c(mime);
        // SAFETY: NUL-terminated mime type.
        Codec::adopt(unsafe { AMediaCodec_createDecoderByType(m.as_ptr()) }, mime)
    }

    pub fn encoder(mime: &str) -> Result<Codec, String> {
        let m = c(mime);
        // SAFETY: NUL-terminated mime type.
        Codec::adopt(unsafe { AMediaCodec_createEncoderByType(m.as_ptr()) }, mime)
    }

    pub fn configure(&mut self, format: &Format, encode: bool) -> Result<(), String> {
        let flags = if encode {
            AMEDIACODEC_CONFIGURE_FLAG_ENCODE as u32
        } else {
            0
        };
        // SAFETY: valid codec and format; no surface, no crypto.
        let status = unsafe {
            AMediaCodec_configure(
                self.ptr,
                format.raw(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                flags,
            )
        };
        ok(status, "Der Codec lehnt die Einstellungen ab")
    }

    pub fn start(&mut self) -> Result<(), String> {
        // SAFETY: valid, configured codec.
        ok(unsafe { AMediaCodec_start(self.ptr) }, "Der Codec startet nicht")?;
        self.started = true;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), String> {
        // SAFETY: valid, started codec.
        ok(unsafe { AMediaCodec_flush(self.ptr) }, "Flush fehlgeschlagen")
    }

    pub fn dequeue_input(&mut self, timeout_us: i64) -> Option<usize> {
        // SAFETY: valid, started codec.
        let i = unsafe { AMediaCodec_dequeueInputBuffer(self.ptr, timeout_us) };
        (i >= 0).then_some(i as usize)
    }

    /// The writable input buffer behind `index` (from [`Codec::dequeue_input`]).
    pub fn input_buffer(&mut self, index: usize) -> &mut [u8] {
        let mut size = 0usize;
        // SAFETY: valid codec; the buffer stays valid until the index is queued, which needs
        // `&mut self` again, so the returned borrow cannot outlive it.
        let p = unsafe { AMediaCodec_getInputBuffer(self.ptr, index, &mut size) };
        if p.is_null() {
            return &mut [];
        }
        // SAFETY: `p` points to `size` writable bytes (see above).
        unsafe { std::slice::from_raw_parts_mut(p, size) }
    }

    pub fn queue_input(
        &mut self,
        index: usize,
        size: usize,
        pts_us: i64,
        flags: u32,
    ) -> Result<(), String> {
        // SAFETY: valid codec; `index` came from dequeue_input.
        let status = unsafe {
            AMediaCodec_queueInputBuffer(self.ptr, index, 0, size, pts_us.max(0) as u64, flags)
        };
        ok(status, "Der Codec nimmt die Daten nicht an")
    }

    pub fn dequeue_output(&mut self, timeout_us: i64) -> Result<Output, String> {
        let mut info = AMediaCodecBufferInfo {
            offset: 0,
            size: 0,
            presentationTimeUs: 0,
            flags: 0,
        };
        // SAFETY: valid codec and info pointer.
        let i = unsafe { AMediaCodec_dequeueOutputBuffer(self.ptr, &mut info, timeout_us) };
        match i {
            i if i >= 0 => Ok(Output::Buffer(OutBuffer {
                index: i as usize,
                size: info.size.max(0) as usize,
                pts: info.presentationTimeUs,
                flags: info.flags,
            })),
            -2 => Ok(Output::FormatChanged),
            -1 | -3 => Ok(Output::TryAgain),
            e => Err(format!("Der Codec meldet Fehler {e}")),
        }
    }

    /// The bytes of an output buffer; empty when the codec hands out nothing.
    pub fn output_bytes(&mut self, out: &OutBuffer) -> Vec<u8> {
        let mut cap = 0usize;
        // SAFETY: valid codec; `out.index` is an output buffer not yet released.
        let p = unsafe { AMediaCodec_getOutputBuffer(self.ptr, out.index, &mut cap) };
        if p.is_null() {
            return Vec::new();
        }
        let n = out.size.min(cap);
        // SAFETY: `p` points to `cap` readable bytes; the offset is zero for the formats we use.
        unsafe { std::slice::from_raw_parts(p, n) }.to_vec()
    }

    pub fn release_output(&mut self, index: usize) {
        // SAFETY: valid codec; the buffer is not touched afterwards.
        unsafe { AMediaCodec_releaseOutputBuffer(self.ptr, index, false) };
    }

    pub fn output_format(&mut self) -> Option<Format> {
        // SAFETY: valid codec.
        Format::adopt(unsafe { AMediaCodec_getOutputFormat(self.ptr) })
    }
}

impl Drop for Codec {
    fn drop(&mut self) {
        // SAFETY: owned; stopped (if running) and deleted once.
        unsafe {
            if self.started {
                AMediaCodec_stop(self.ptr);
            }
            AMediaCodec_delete(self.ptr);
        }
    }
}

// ----------------------------------------------------------------- MediaMuxer

pub struct Muxer {
    ptr: *mut AMediaMuxer,
    started: bool,
    // The muxer writes through this descriptor: keep the file open as long as it lives.
    _file: File,
}

// SAFETY: the muxer is only ever used by the thread that currently owns the value.
unsafe impl Send for Muxer {}

impl Muxer {
    pub fn create(path: &Path) -> Result<Muxer, String> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| format!("Datei lässt sich nicht anlegen: {e}"))?;
        // SAFETY: the descriptor is valid and read/write, as the muxer requires.
        let ptr = unsafe {
            AMediaMuxer_new(
                file.as_raw_fd(),
                OutputFormat::AMEDIAMUXER_OUTPUT_FORMAT_MPEG_4,
            )
        };
        if ptr.is_null() {
            return Err("Der MP4-Schreiber lässt sich nicht anlegen".into());
        }
        Ok(Muxer {
            ptr,
            started: false,
            _file: file,
        })
    }

    pub fn add_track(&mut self, format: &Format) -> Result<usize, String> {
        // SAFETY: valid muxer (not yet started) and format.
        let i = unsafe { AMediaMuxer_addTrack(self.ptr, format.raw()) };
        if i < 0 {
            Err("Spur lässt sich nicht hinzufügen".into())
        } else {
            Ok(i as usize)
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        // SAFETY: valid muxer with all tracks added.
        ok(unsafe { AMediaMuxer_start(self.ptr) }, "MP4 startet nicht")?;
        self.started = true;
        Ok(())
    }

    pub fn write(&mut self, track: usize, data: &[u8], pts_us: i64, flags: u32) -> Result<(), String> {
        let info = AMediaCodecBufferInfo {
            offset: 0,
            size: data.len() as i32,
            presentationTimeUs: pts_us.max(0),
            flags,
        };
        // SAFETY: started muxer; `data` and `info` are valid for the call.
        let status = unsafe { AMediaMuxer_writeSampleData(self.ptr, track, data.as_ptr(), &info) };
        ok(status, "Schreiben fehlgeschlagen")
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if !self.started {
            return Ok(());
        }
        self.started = false;
        // SAFETY: started muxer.
        ok(unsafe { AMediaMuxer_stop(self.ptr) }, "MP4 lässt sich nicht abschließen")
    }
}

impl Drop for Muxer {
    fn drop(&mut self) {
        // SAFETY: owned; stopped first if still running, deleted once.
        unsafe {
            if self.started {
                AMediaMuxer_stop(self.ptr);
            }
            AMediaMuxer_delete(self.ptr);
        }
    }
}
