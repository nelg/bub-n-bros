//! Sound effects and background music (see display/snd_pygame.py).

use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
                       PlaySoundParams, Sound};
use crate::files::DataStore;
use std::collections::HashMap;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// Run a future that is known to complete immediately (sound loading is
/// synchronous on desktop platforms, the async API is only for the web).
fn now<F: Future>(f: F) -> Option<F::Output> {
    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    match pin!(f).poll(&mut cx) {
        Poll::Ready(v) => Some(v),
        Poll::Pending => None,
    }
}

/// Information from a WAV header.
pub struct WavInfo {
    /// total size of the file according to its RIFF header
    pub file_size: usize,
    /// duration in seconds
    pub duration: f64,
}

pub fn wav_info(data: &[u8]) -> Option<WavInfo> {
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }
    let file_size = u32::from_le_bytes(data[4..8].try_into().ok()?) as usize + 8;
    let mut pos = 12;
    let mut byte_rate = 0u32;
    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().ok()?) as usize;
        if id == b"fmt " && pos + 20 <= data.len() {
            byte_rate = u32::from_le_bytes(data[pos + 16..pos + 20].try_into().ok()?);
        } else if id == b"data" {
            let duration = if byte_rate > 0 { size as f64 / byte_rate as f64 } else { 0.0 };
            return Some(WavInfo { file_size, duration });
        }
        pos += 8 + size + (size & 1);
    }
    None
}

pub fn load_wav(data: &[u8]) -> Option<Sound> {
    let info = wav_info(data)?;
    if info.duration <= 0.0 {
        return None;
    }
    // the decoder panics on broken files: be careful
    std::panic::catch_unwind(|| now(load_sound_from_bytes(data)))
        .ok()
        .flatten()
        .and_then(|r| r.ok())
}

/// The volume can go above 100% (louder, at the risk of some distortion).
pub const MAX_VOLUME: f32 = 2.0;

/// Our ears hear loudness roughly logarithmically: a linear gain from 100%
/// to 80% is barely audible.  Square the volume setting so that each step
/// makes a clear difference (80% -> 0.64, 60% -> 0.36, 200% -> 4.0).
fn gain(volume: f32) -> f32 {
    volume * volume
}

/// Music that is still downloading is played in pieces of this length.
const SEGMENT_SECONDS: usize = 12;
const FIRST_SEGMENT_SECONDS: usize = 3;

/// Layout of a PCM WAV file: where its data is and how to rebuild a header.
#[derive(Clone)]
struct WavLayout {
    fmt: Vec<u8>,       // the "fmt " chunk, header included
    data_start: usize,  // offset of the sample data in the file
    data_len: usize,
    byte_rate: usize,
    block_align: usize,
}

fn wav_layout(head: &[u8]) -> Option<WavLayout> {
    if head.len() < 12 || &head[0..4] != b"RIFF" || &head[8..12] != b"WAVE" {
        return None;
    }
    let mut pos = 12;
    let mut fmt = None;
    while pos + 8 <= head.len() {
        let id = &head[pos..pos + 4];
        let size = u32::from_le_bytes(head[pos + 4..pos + 8].try_into().ok()?) as usize;
        if id == b"fmt " && pos + 8 + 16 <= head.len() {
            fmt = Some(head[pos..pos + 8 + size.min(head.len() - pos - 8)].to_vec());
        } else if id == b"data" {
            let fmt = fmt?;
            let byte_rate = u32::from_le_bytes(fmt[16..20].try_into().ok()?) as usize;
            let block_align = (u16::from_le_bytes(fmt[20..22].try_into().ok()?) as usize).max(1);
            return Some(WavLayout { fmt, data_start: pos + 8, data_len: size, byte_rate, block_align });
        }
        pos += 8 + size + (size & 1);
    }
    None
}

/// A WAV file containing `samples` with the format of `layout`.
fn make_wav(layout: &WavLayout, samples: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() + 64);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&((4 + layout.fmt.len() + 8 + samples.len()) as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(&layout.fmt);
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(samples.len() as u32).to_le_bytes());
    out.extend_from_slice(samples);
    out
}

/// Music being played: which file, how far, and until when.
struct Playing {
    index: usize,      // in the playlist
    next_pos: usize,   // offset in the file's sample data of the next piece
    whole: bool,       // the whole file is being played in one go
    end: f64,          // when the current piece ends
    sound: Sound,
}

pub struct Audio {
    pub enabled: bool,
    pub music_enabled: bool,
    /// sample code -> sound
    pub samples: HashMap<i64, Sound>,
    /// music file id -> whole sound, once completely downloaded
    pub musics: HashMap<i64, Sound>,
    /// current music: list of file ids and index to loop from
    pub playlist: Vec<i64>,
    pub loop_from: usize,
    playing: Option<Playing>,
    /// the previous piece, kept alive until it has finished
    previous: Option<Sound>,
    pub fade: Option<(f64, f64)>,      // (start time, duration)
    /// volume of the sound effects and of the music (1.0 = normal)
    pub volume: f32,
    pub music_volume: f32,
    pub muted: bool,
}

impl Audio {
    pub fn new(enabled: bool, music_enabled: bool) -> Audio {
        Audio {
            enabled,
            music_enabled: enabled && music_enabled,
            samples: HashMap::new(),
            musics: HashMap::new(),
            playlist: Vec::new(),
            loop_from: 0,
            playing: None,
            previous: None,
            fade: None,
            volume: 1.0,
            music_volume: 1.0,
            muted: false,
        }
    }

    /// Mute or unmute everything (sound effects and music).
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if self.fade.is_none() {
            if let Some(p) = &self.playing {
                set_sound_volume(&p.sound, self.effective_music_volume());
            }
        }
    }

    pub fn play_sample(&self, code: i64, lvol: f32, rvol: f32) {
        if self.muted {
            return;
        }
        if let Some(s) = self.samples.get(&code) {
            // The server gives a left and a right volume (stereo position).
            // We play in mono, so use the louder side: averaging them made
            // sounds near the sides of the board half as loud.
            let volume = lvol.max(rvol).clamp(0.0, 1.0) * gain(self.volume);
            play_sound(s, PlaySoundParams { looped: false, volume });
        }
    }

    /// Short description of what music is playing, for statistics.
    pub fn music_status(&self) -> String {
        match &self.playing {
            None => "no music playing".into(),
            Some(p) if p.whole => format!("playing music {} (whole file)", p.index),
            Some(p) => format!("playing music {} (streaming, {} kbytes in)", p.index, p.next_pos / 1024),
        }
    }

    fn effective_music_volume(&self) -> f32 {
        if self.muted { 0.0 } else { gain(self.volume * self.music_volume) }
    }

    /// Change the master volume (sound effects and music).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, MAX_VOLUME);
        if self.fade.is_none() {
            if let Some(p) = &self.playing {
                set_sound_volume(&p.sound, self.effective_music_volume());
            }
        }
    }

    fn stop_current(&mut self) {
        if let Some(p) = self.playing.take() {
            stop_sound(&p.sound);
        }
        if let Some(s) = self.previous.take() {
            stop_sound(&s);
        }
    }

    pub fn play_musics(&mut self, playlist: Vec<i64>, loop_from: usize) {
        self.stop_current();
        self.fade = None;
        self.playlist = playlist;
        self.loop_from = loop_from;
    }

    pub fn fadeout(&mut self, millisec: f64, time: f64) {
        self.fade = Some((time, millisec / 1000.0));
    }

    /// Called every frame: start the next piece of music when needed.
    ///
    /// The server streams the music files about one second ahead of
    /// playing them, so a file is usually not complete when it must start.
    /// Complete files are played in one go; others are played in pieces of
    /// SEGMENT_SECONDS, each started when the previous one ends and its
    /// data has arrived.
    pub fn update(&mut self, time: f64, store: &DataStore) {
        if !self.music_enabled {
            return;
        }
        if let Some((start, length)) = self.fade {
            let f = 1.0 - (time - start) / length.max(0.001);
            if let Some(p) = &self.playing {
                set_sound_volume(&p.sound, f.max(0.0) as f32 * self.effective_music_volume());
            }
            if f <= 0.0 {
                self.stop_current();
                self.playlist.clear();
                self.fade = None;
            }
            return;
        }
        // start the next piece a moment before the current one ends, to
        // avoid a gap (frames come every ~16 ms)
        let (index, pos) = match &self.playing {
            None => {
                if self.playlist.is_empty() {
                    return;
                }
                (0, 0)
            }
            Some(p) => {
                if time < p.end - 0.012 {
                    return;
                }
                let id = self.playlist[p.index];
                let len = store.files.get(&id)
                    .and_then(|f| f.slice(0, 256))
                    .and_then(|h| wav_layout(&h))
                    .map_or(0, |l| l.data_len);
                if !p.whole && p.next_pos < len {
                    (p.index, p.next_pos)
                } else if p.index + 1 < self.playlist.len() {
                    (p.index + 1, 0)
                } else {
                    (self.loop_from, 0)
                }
            }
        };
        if index >= self.playlist.len() {
            self.stop_current();
            self.playlist.clear();
            return;
        }
        let id = self.playlist[index];
        let volume = self.effective_music_volume();

        // the whole file, if we have it
        if pos == 0 {
            if !self.musics.contains_key(&id) {
                if let Some(f) = store.files.get(&id) {
                    if let Some(info) = f.slice(0, 256).and_then(|h| wav_info(&h)) {
                        if f.pending.is_empty() && f.contiguous_len() >= info.file_size {
                            if let Some(s) = f.slice(0, info.file_size).and_then(|d| load_wav(&d)) {
                                self.musics.insert(id, s);
                            }
                        }
                    }
                }
            }
            if let Some(s) = self.musics.get(&id) {
                let duration = store.files.get(&id)
                    .and_then(|f| f.slice(0, 256))
                    .and_then(|h| wav_info(&h))
                    .map_or(1.0, |i| i.duration);
                play_sound(s, PlaySoundParams { looped: false, volume });
                self.previous = self.playing.take().map(|p| p.sound);
                self.playing = Some(Playing { index, next_pos: 0, whole: true,
                                              end: time + duration, sound: s.clone() });
                return;
            }
        }

        // otherwise the next piece, once it has been downloaded
        let f = match store.files.get(&id) {
            Some(f) => f,
            None => return,
        };
        let layout = match f.slice(0, 256).and_then(|h| wav_layout(&h)) {
            Some(l) => l,
            None => return,
        };
        // a short first piece, so that the music starts soon
        let seconds = if pos == 0 { FIRST_SEGMENT_SECONDS } else { SEGMENT_SECONDS };
        let mut seg = layout.byte_rate * seconds;
        seg -= seg % layout.block_align;
        let end = (pos + seg).min(layout.data_len);
        if end <= pos {
            return;
        }
        let samples = match f.slice(layout.data_start + pos, layout.data_start + end) {
            Some(s) => s,
            None => return, // not downloaded yet: try again next frame
        };
        if let Some(s) = load_wav(&make_wav(&layout, &samples)) {
            play_sound(&s, PlaySoundParams { looped: false, volume });
            self.previous = self.playing.take().map(|p| p.sound);
            self.playing = Some(Playing {
                index,
                next_pos: end,
                whole: false,
                end: time + (end - pos) as f64 / layout.byte_rate.max(1) as f64,
                sound: s,
            });
        }
    }
}
