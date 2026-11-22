//! Sound effects and background music (see display/snd_pygame.py).

use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
                       PlaySoundParams, Sound};
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
pub const MAX_VOLUME: f32 = 3.0;

pub struct Audio {
    pub enabled: bool,
    pub music_enabled: bool,
    /// sample code -> sound
    pub samples: HashMap<i64, Sound>,
    /// music file id -> (sound, duration)
    pub musics: HashMap<i64, (Sound, f64)>,
    /// current music: list of file ids, index to loop from, index playing
    pub playlist: Vec<i64>,
    pub loop_from: usize,
    pub current: Option<(usize, f64)>, // (index, end time)
    pub fade: Option<(f64, f64)>,      // (start time, duration)
    /// volume of the sound effects and of the music (1.0 = normal)
    pub volume: f32,
    pub music_volume: f32,
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
            current: None,
            fade: None,
            volume: 1.0,
            music_volume: 1.0,
        }
    }

    pub fn play_sample(&self, code: i64, lvol: f32, rvol: f32) {
        if let Some(s) = self.samples.get(&code) {
            // The server gives a left and a right volume (stereo position).
            // We play in mono, so use the louder side: averaging them made
            // sounds near the sides of the board half as loud.
            let volume = lvol.max(rvol).clamp(0.0, 1.0) * self.volume;
            play_sound(s, PlaySoundParams { looped: false, volume });
        }
    }

    fn effective_music_volume(&self) -> f32 {
        self.volume * self.music_volume
    }

    /// Change the master volume (sound effects and music).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, MAX_VOLUME);
        if self.fade.is_none() {
            if let Some((i, _)) = self.current {
                if let Some((s, _)) = self.playlist.get(i).and_then(|id| self.musics.get(id)) {
                    set_sound_volume(s, self.effective_music_volume());
                }
            }
        }
    }

    fn stop_current(&mut self) {
        if let Some((i, _)) = self.current.take() {
            if let Some(id) = self.playlist.get(i) {
                if let Some((s, _)) = self.musics.get(id) {
                    stop_sound(s);
                }
            }
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
    pub fn update(&mut self, time: f64) {
        if !self.music_enabled {
            return;
        }
        if let Some((start, length)) = self.fade {
            let f = 1.0 - (time - start) / length.max(0.001);
            if let Some((i, _)) = self.current {
                if let Some((s, _)) = self.playlist.get(i).and_then(|id| self.musics.get(id)) {
                    set_sound_volume(s, f.max(0.0) as f32 * self.effective_music_volume());
                }
            }
            if f <= 0.0 {
                self.stop_current();
                self.playlist.clear();
                self.fade = None;
            }
            return;
        }
        let next = match self.current {
            None => {
                if self.playlist.is_empty() { return; }
                0
            }
            Some((i, end)) => {
                if time < end { return; }
                if i + 1 < self.playlist.len() { i + 1 } else { self.loop_from }
            }
        };
        if next >= self.playlist.len() {
            self.current = None;
            self.playlist.clear();
            return;
        }
        // wait until every piece of the playlist has been downloaded
        if self.playlist.iter().any(|id| !self.musics.contains_key(id)) {
            return;
        }
        let (s, duration) = &self.musics[&self.playlist[next]];
        play_sound(s, PlaySoundParams { looped: false, volume: self.effective_music_volume() });
        self.current = Some((next, time + duration));
    }
}
