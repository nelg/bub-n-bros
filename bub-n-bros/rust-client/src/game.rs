//! The client side of the game: protocol handling, sprites, taskbar and
//! player keys.  This is a port of display/pclient.py.

use crate::audio::{self, Audio};
use crate::files::DataStore;
use crate::proto::*;
use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress};
use macroquad::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};

pub const TASKBAR_HEIGHT: f32 = 48.0;
/// Switch to frames over TCP if UDP delivers less than this ratio.
const UDP_EXPECTED_RATIO: f64 = 0.60;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayState {
    Off,
    Remote,
    Local,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UdpMode {
    Auto,
    Udp,
    Tcp,
}

enum BitmapSource {
    Inline(Vec<u8>), // zlib-compressed PPM
    File(i64),       // file id in the DataStore
}

struct Bitmap {
    source: BitmapSource,
    colorkey: Option<i64>,
    texture: Option<Texture2D>,
}

#[derive(Clone)]
struct Icon {
    bmp: i64,
    rect: Rect,
    alpha: u8,
}

#[derive(Default)]
struct DynThread {
    decomp: Option<Decompress>,
    lastframe: Option<u8>,
    recompressed: bool,
    data: Vec<u8>,
}

pub struct Options {
    pub udp_mode: UdpMode,
    pub sound: bool,
    pub music: bool,
    pub names: HashMap<i64, String>,
    pub default_name: Option<String>,
    pub volume: f32,
    pub music_volume: f32,
}

pub struct Game {
    tcp: TcpStream,
    server: SocketAddr,
    inbuf: Vec<u8>,
    outbuf: Vec<u8>,
    pub gameident: String,
    pub closed: Option<String>,
    opts: Options,

    pub width: f32,
    pub height: f32,
    pub defined: bool, // got MSG_DEF_PLAYFIELD
    initlevel: u8,

    bitmaps: HashMap<i64, Bitmap>,
    icons: HashMap<i64, Icon>,
    pub data: DataStore,
    pending_samples: HashMap<i64, i64>, // sample code -> file id
    audio: Audio,

    // frames
    udp: Option<UdpSocket>,
    inline: Option<Decompress>,
    udpsock_low: Option<u32>,
    udpbytes: u64,
    dyn_threads: Vec<DynThread>,
    dyn_norepeat: Option<u8>,
    frame: Vec<u8>,
    playingsounds: HashMap<i64, u32>,
    pub frames_received: u64,

    // players and keys
    playing: HashMap<i64, PlayState>,
    keys: BTreeMap<String, (i64, Vec<i64>)>,
    keycodes: HashMap<(KeyCode, bool), (i64, Vec<u8>)>,
    keydefinition: Option<(i64, HashMap<String, KeyCode>)>,
    playericons: BTreeMap<i64, i64>,
    taskbarmode: bool,
    taskbarfree: bool,
    taskbartimeout: Option<f64>,
    pub animated: bool,
}

fn decode_ppm(data: &[u8]) -> Option<(u16, u16, &[u8])> {
    // "P6\n<w> <h>\n255\n<data>", possibly with comment lines
    let mut fields = Vec::new();
    let mut pos = 0;
    while fields.len() < 4 {
        while pos < data.len() && data[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos < data.len() && data[pos] == b'#' {
            while pos < data.len() && data[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }
        let start = pos;
        while pos < data.len() && !data[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if start == pos {
            return None;
        }
        fields.push(std::str::from_utf8(&data[start..pos]).ok()?.to_string());
    }
    pos += 1; // the single whitespace after "255"
    if fields[0] != "P6" {
        return None;
    }
    let w: u16 = fields[1].parse().ok()?;
    let h: u16 = fields[2].parse().ok()?;
    let len = w as usize * h as usize * 3;
    if data.len() < pos + len {
        return None;
    }
    Some((w, h, &data[pos..pos + len]))
}

fn inflate(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(data).read_to_end(&mut out).ok()?;
    Some(out)
}

/// Decompress a chunk of a continuous zlib stream (Z_SYNC_FLUSH'ed).
fn inflate_stream(d: &mut Decompress, data: &[u8]) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(data.len() * 8 + 4096);
    let mut input = data;
    loop {
        if out.len() == out.capacity() {
            out.reserve(out.capacity().max(8192));
        }
        let before_in = d.total_in();
        let before_out = d.total_out();
        d.decompress_vec(input, &mut out, FlushDecompress::Sync).ok()?;
        let consumed = (d.total_in() - before_in) as usize;
        let produced = (d.total_out() - before_out) as usize;
        input = &input[consumed..];
        if input.is_empty() && out.len() < out.capacity() {
            break; // everything decoded and the output buffer was not full
        }
        if consumed == 0 && produced == 0 {
            break; // no progress
        }
    }
    Some(out)
}

impl Game {
    pub fn new(mut tcp: TcpStream, opts: Options, cachedir: Option<std::path::PathBuf>)
               -> std::io::Result<Game> {
        let server = tcp.peer_addr()?;
        let _ = tcp.set_nodelay(true);
        // read the welcome line and the game name
        let mut buf = Vec::new();
        let mut chunk = [0u8; 256];
        loop {
            let n = tcp.read(&mut chunk)?;
            if n == 0 {
                return Err(std::io::Error::new(ErrorKind::UnexpectedEof, "connexion closed"));
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.len() >= MSG_WELCOME.len() {
                if &buf[..MSG_WELCOME.len()] != MSG_WELCOME {
                    return Err(std::io::Error::new(ErrorKind::InvalidData,
                                                   "connected to something not a game server"));
                }
                if buf[MSG_WELCOME.len()..].contains(&b'\n') {
                    break;
                }
            }
        }
        let tail = &buf[MSG_WELCOME.len()..];
        let n = tail.iter().position(|&c| c == b'\n').unwrap();
        let gameident = String::from_utf8_lossy(&tail[..n]).trim().to_string();
        let inbuf = tail[n + 1..].to_vec();
        eprintln!("connected to {:?}.", gameident);
        tcp.set_nonblocking(true)?;

        let mut audio = Audio::new(opts.sound, opts.music);
        audio.set_volume(opts.volume);
        audio.music_volume = opts.music_volume;
        let udpsock_low = if opts.udp_mode == UdpMode::Auto { Some(0) } else { None };
        let mut game = Game {
            tcp, server, inbuf, outbuf: Vec::new(), gameident, closed: None,
            width: 640.0, height: 480.0, defined: false, initlevel: 0,
            bitmaps: HashMap::new(), icons: HashMap::new(),
            data: DataStore::new(cachedir), pending_samples: HashMap::new(), audio,
            udp: None, inline: None, udpsock_low, udpbytes: 0,
            dyn_threads: (0..8).map(|_| DynThread::default()).collect(),
            dyn_norepeat: None, frame: Vec::new(), playingsounds: HashMap::new(),
            frames_received: 0,
            playing: HashMap::new(), keys: BTreeMap::new(), keycodes: HashMap::new(),
            keydefinition: None, playericons: BTreeMap::new(),
            taskbarmode: false, taskbarfree: false, taskbartimeout: None, animated: false,
            opts,
        };
        if game.opts.udp_mode == UdpMode::Tcp {
            game.start_udp_over_tcp();
        }
        game.send(message(CMSG_PROTO_VERSION, &[3.into()]));
        Ok(game)
    }

    // ------------------------------------------------------------------
    // networking

    fn send(&mut self, msg: Vec<u8>) {
        self.outbuf.extend_from_slice(&msg);
        self.flush();
    }

    fn flush(&mut self) {
        while !self.outbuf.is_empty() {
            match self.tcp.write(&self.outbuf) {
                Ok(0) => break,
                Ok(n) => {
                    self.outbuf.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => {
                    self.closed = Some(format!("send: {}", e));
                    break;
                }
            }
        }
    }

    /// Read and handle everything the server sent.  Called once per frame.
    pub fn poll(&mut self, time: f64) {
        self.flush();
        let mut chunk = vec![0u8; 0x10000];
        loop {
            match self.tcp.read(&mut chunk) {
                Ok(0) => {
                    self.closed = Some("connexion closed by the server".into());
                    break;
                }
                Ok(n) => self.inbuf.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    self.closed = Some(format!("recv: {}", e));
                    break;
                }
            }
        }
        let mut pos = 0;
        loop {
            match decode(&self.inbuf[pos..]) {
                Decoded::Message(tp, values, used) => {
                    pos += used;
                    self.handle(tp, values, time);
                }
                Decoded::Incomplete => break,
                Decoded::Error => {
                    self.closed = Some("protocol error".into());
                    break;
                }
            }
        }
        self.inbuf.drain(..pos);

        // UDP frames
        let mut latest = None;
        if let Some(udp) = &self.udp {
            let mut buf = vec![0u8; 65536];
            let mut packets = Vec::new();
            loop {
                match udp.recv(&mut buf) {
                    Ok(n) => packets.push(buf[..n].to_vec()),
                    Err(_) => break,
                }
            }
            for p in packets {
                self.udpbytes += p.len() as u64;
                let frame = if p.len() > 3 && (0x80..0x90).contains(&p[0]) {
                    self.dynamic_decompress(&p)
                } else {
                    Some(p)
                };
                if frame.is_some() {
                    latest = frame;
                }
            }
        }
        if let Some(f) = latest {
            self.update_frame(f);
        }
        self.load_ready_data();
        self.prefetch_music();
        self.audio.update(time, &self.data);
        if let Some(t) = self.taskbartimeout {
            if time > t {
                self.settaskbar(false, time);
            }
        }
        self.flush();
    }

    /// Protocol version 3 UDP frames: zlib streams on 8 rotating
    /// "threads" with sync points, so that lost packets only spoil a few
    /// frames.  See gamesrv.Client.setup_dyncompress() and
    /// pclient.Playfield.dynamic_decompress().
    fn dynamic_decompress(&mut self, packet: &[u8]) -> Option<Vec<u8>> {
        let t = (packet[0] & 7) as usize;
        let prevframe = packet[1];
        let thisframe = packet[2];
        if packet[0] >= 0x88 {
            // reset
            let mut d = Decompress::new(true);
            if prevframe != thisframe {
                // sync point from a previous frame: prime the decompressor
                // with that frame's data
                let candidates: Vec<usize> = (0..8)
                    .filter(|&i| self.dyn_threads[i].lastframe == Some(prevframe))
                    .collect();
                if candidates.is_empty() {
                    return None;
                }
                let idx = candidates.iter().copied()
                    .find(|&i| self.dyn_threads[i].recompressed)
                    .unwrap_or(candidates[0]);
                if !self.dyn_threads[idx].recompressed {
                    let raw = std::mem::take(&mut self.dyn_threads[idx].data);
                    let mut c = Compress::new(Compression::new(6), true);
                    let mut out = Vec::with_capacity(raw.len() + 1024);
                    c.compress_vec(&raw, &mut out, FlushCompress::Sync).ok()?;
                    self.dyn_threads[idx].data = out;
                    self.dyn_threads[idx].recompressed = true;
                }
                let primer = self.dyn_threads[idx].data.clone();
                inflate_stream(&mut d, &primer)?;
            }
            self.dyn_threads[t].decomp = Some(d);
        } else if self.dyn_threads[t].lastframe != Some(prevframe)
            || self.dyn_threads[t].decomp.is_none()
        {
            return None; // lost
        }
        let th = &mut self.dyn_threads[t];
        let framedata = inflate_stream(th.decomp.as_mut().unwrap(), &packet[3..])?;
        th.lastframe = Some(thisframe);
        th.recompressed = false;
        th.data = framedata.clone();
        if self.dyn_norepeat == Some(thisframe) {
            return None;
        }
        self.dyn_norepeat = Some(thisframe);
        Some(framedata)
    }

    fn start_udp_over_tcp(&mut self) {
        self.inline = Some(Decompress::new(true));
        self.udpsock_low = None;
        self.udp = None;
    }

    fn start_playing(&mut self) {
        let port: Value = if self.inline.is_some() {
            Value::Bytes(vec![MSG_INLINE_FRAME])
        } else {
            match UdpSocket::bind("0.0.0.0:0") {
                Ok(s) => {
                    let _ = s.set_nonblocking(true);
                    // a first packet helps some NATs let the answers through
                    let _ = s.send_to(b".", self.server);
                    let port = s.local_addr().map(|a| a.port()).unwrap_or(0);
                    self.udp = Some(s);
                    Value::Int(port as i64)
                }
                Err(e) => {
                    eprintln!("cannot open an UDP socket ({}), using TCP", e);
                    self.start_udp_over_tcp();
                    Value::Bytes(vec![MSG_INLINE_FRAME])
                }
            }
        };
        let mut out = message(CMSG_UDP_PORT, &[port]);
        if self.audio.music_enabled {
            out.extend(message(CMSG_ENABLE_MUSIC, &[1.into()]));
            out.extend(message(CMSG_PING, &[]));
        }
        self.send(out);
    }

    // ------------------------------------------------------------------
    // messages

    fn handle(&mut self, tp: u8, v: Vec<Value>, time: f64) {
        let int = |i: usize| v.get(i).and_then(|x| x.int());
        match tp {
            MSG_DEF_PLAYFIELD => {
                self.width = int(0).unwrap_or(640) as f32;
                self.height = int(1).unwrap_or(480) as f32;
                if let Some(g) = v.get(3) {
                    self.gameident = g.text();
                }
                self.defined = true;
                if self.audio.enabled {
                    self.send(message(CMSG_ENABLE_SOUND, &[]));
                }
                self.send(message(CMSG_PING, &[]));
                self.taskbarmode = false;
                self.taskbarfree = false;
                self.taskbartimeout = None;
                self.keydefinition = None;
            }
            MSG_DEF_KEY => {
                if let (Some(name), Some(num)) = (v.first(), int(1)) {
                    let icons = v[2..].iter().filter_map(|x| x.int()).collect();
                    self.keys.insert(name.text(), (num, icons));
                }
            }
            MSG_DEF_ICON => {
                if v.len() >= 6 {
                    let alpha = int(6).unwrap_or(255).clamp(0, 255) as u8;
                    self.icons.insert(int(1).unwrap(), Icon {
                        bmp: int(0).unwrap(),
                        rect: Rect::new(int(2).unwrap() as f32, int(3).unwrap() as f32,
                                        int(4).unwrap() as f32, int(5).unwrap() as f32),
                        alpha,
                    });
                }
            }
            MSG_DEF_BITMAP => {
                let code = int(0).unwrap_or(-1);
                let source = match v.get(1) {
                    Some(Value::Bytes(b)) => BitmapSource::Inline(b.clone()),
                    Some(Value::Int(f)) => BitmapSource::File(*f),
                    None => return,
                };
                self.bitmaps.insert(code, Bitmap { source, colorkey: int(2), texture: None });
            }
            MSG_DEF_SAMPLE => {
                let code = int(0).unwrap_or(-1);
                match v.get(1) {
                    Some(Value::Bytes(b)) => {
                        if self.audio.enabled {
                            if let Some(s) = inflate(b).and_then(|d| audio::load_wav(&d)) {
                                self.audio.samples.insert(code, s);
                            }
                        }
                    }
                    Some(Value::Int(f)) => {
                        self.pending_samples.insert(code, *f);
                    }
                    None => {}
                }
            }
            MSG_PLAY_MUSIC => {
                let loop_from = int(0).unwrap_or(0).max(0) as usize;
                let codes = v[1..].iter().filter_map(|x| x.int()).collect();
                self.audio.play_musics(codes, loop_from);
            }
            MSG_FADEOUT => {
                self.audio.fadeout(int(0).unwrap_or(1000) as f64, time);
            }
            MSG_PLAYER_JOIN => {
                let (id, local) = (int(0).unwrap_or(-1), int(1).unwrap_or(0));
                if local != 0 {
                    self.playing.insert(id, PlayState::Local);
                    self.settaskbar(false, time);
                    let name = self.opts.names.get(&id).cloned()
                        .or_else(|| self.opts.default_name.clone());
                    if let Some(name) = name {
                        self.send(message(CMSG_PLAYER_NAME, &[id.into(), name.as_str().into()]));
                    }
                } else {
                    self.playing.insert(id, PlayState::Remote);
                }
            }
            MSG_PLAYER_KILL => {
                let id = int(0).unwrap_or(-1);
                self.playing.insert(id, PlayState::Off);
                self.keycodes.retain(|_, (pid, _)| *pid != id);
            }
            MSG_PLAYER_ICON => {
                if let (Some(pid), Some(ico)) = (int(0), int(1)) {
                    self.playericons.insert(pid, ico);
                }
            }
            MSG_PING => {
                self.send(message(CMSG_PONG, &v));
                if let (Some(low), Some(kbytes)) = (self.udpsock_low, int(0)) {
                    if kbytes == 0 {
                        return;
                    }
                    if ((kbytes << 10) as f64) * UDP_EXPECTED_RATIO > self.udpbytes as f64 {
                        // too many packets were dropped (maybe all of them)
                        let low = low + 1;
                        self.udpsock_low = Some(low);
                        if low >= 3 && self.initlevel >= 1 {
                            eprintln!("Note: routing UDP traffic over TCP");
                            self.start_udp_over_tcp();
                            self.send(message(CMSG_UDP_PORT, &[Value::Bytes(vec![MSG_INLINE_FRAME])]));
                        }
                    } else {
                        self.udpsock_low = Some(0);
                    }
                }
            }
            MSG_PONG => {
                if self.initlevel == 0 {
                    self.start_playing();
                    self.initlevel = 1;
                } else if self.initlevel == 1 {
                    if self.audio.music_enabled {
                        self.send(message(CMSG_ENABLE_MUSIC, &[2.into()]));
                    }
                    self.initlevel = 2;
                }
                if !self.taskbarfree && !self.taskbarmode {
                    self.taskbarfree = true;
                    self.settaskbar(true, time);
                }
            }
            MSG_INLINE_FRAME => {
                if let (Some(d), Some(data)) = (self.inline.as_mut(), v.first().and_then(|x| x.bytes())) {
                    if let Some(frame) = inflate_stream(d, data) {
                        self.update_frame(frame);
                    }
                }
            }
            MSG_PATCH_FILE => {
                if let (Some(fid), Some(pos), Some(data)) = (int(0), int(1), v.get(2).and_then(|x| x.bytes())) {
                    let n = int(3).unwrap_or(data.len() as i64) as usize;
                    self.data.patch(fid, pos as usize, data.to_vec(), n);
                }
            }
            MSG_ZPATCH_FILE => {
                if let (Some(fid), Some(pos), Some(z)) = (int(0), int(1), v.get(2).and_then(|x| x.bytes())) {
                    if let Some(data) = inflate(z) {
                        self.data.patch(fid, pos as usize, data, z.len());
                    }
                }
            }
            MSG_MD5_FILE => {
                if v.len() >= 5 {
                    let fid = int(0).unwrap();
                    let filename = v[1].text();
                    let (pos, len) = (int(2).unwrap() as usize, int(3).unwrap() as usize);
                    let sum = v[4].bytes().unwrap_or(&[]).to_vec();
                    if self.data.md5_file(fid, &filename, pos, len, &sum) {
                        self.send(message(CMSG_DATA_REQUEST,
                                          &[fid.into(), (pos as i64).into(), (len as i64).into()]));
                    }
                }
            }
            MSG_BROADCAST_PORT => {} // LAN broadcast of frames: not used
            _ => {}
        }
    }

    /// The server streams music at playing speed (one second of sound per
    /// second), which is slow for a tune that must start now.  As soon as
    /// we know a music file's size (from its first block), we ask for the
    /// rest ourselves, a few blocks at a time.  The blocks are aligned with
    /// the server's own (one second each), so when the server later sends
    /// their MD5, they are found in the cache and not sent twice.
    fn prefetch_music(&mut self) {
        const BLOCK: usize = 44100;       // gamesrv.Music filerate
        const IN_FLIGHT: usize = 12;      // requests outstanding per file
        if !self.audio.music_enabled {
            return;
        }
        let mut requests = Vec::new();
        for &id in &self.audio.playlist {
            let f = match self.data.files.get_mut(&id) {
                Some(f) => f,
                None => continue,
            };
            let size = match f.slice(0, 64).and_then(|h| audio::wav_info(&h)) {
                Some(info) => info.file_size,
                None => continue,
            };
            let mut pos = 0;
            while pos < size && f.pending.len() < IN_FLIGHT {
                let end = (pos + BLOCK).min(size);
                if !f.covers(pos, end) && !f.pending.contains(&pos) {
                    f.pending.push(pos);
                    requests.push((id, pos, end - pos));
                }
                pos += BLOCK;
            }
            if f.pending.len() >= IN_FLIGHT {
                break; // finish this tune before the next one
            }
        }
        for (id, pos, len) in requests {
            self.send(message(CMSG_DATA_REQUEST, &[id.into(), (pos as i64).into(), (len as i64).into()]));
        }
    }

    /// Turn downloaded files into textures and sounds once they are complete.
    fn load_ready_data(&mut self) {
        let ready: Vec<(i64, i64)> = self.pending_samples.iter()
            .filter(|(_, f)| self.data.files.get(f).map_or(false, |d| d.ready()))
            .map(|(&c, &f)| (c, f))
            .collect();
        for (code, fid) in ready {
            self.pending_samples.remove(&code);
            if self.audio.enabled {
                let data = self.data.files[&fid].read();
                if let Some(s) = audio::load_wav(&data) {
                    self.audio.samples.insert(code, s);
                }
            }
        }
    }

    fn texture(&mut self, bmp: i64) -> Option<Texture2D> {
        let b = self.bitmaps.get_mut(&bmp)?;
        if let Some(t) = &b.texture {
            return Some(t.clone());
        }
        let raw = match &b.source {
            BitmapSource::Inline(z) => inflate(z)?,
            BitmapSource::File(fid) => {
                let f = self.data.files.get(fid)?;
                if !f.ready() {
                    return None;
                }
                f.read()
            }
        };
        let (w, h, rgb) = decode_ppm(&raw)?;
        let key = b.colorkey.filter(|&k| k >= 0).map(|k| {
            [(k & 0xFF) as u8, ((k >> 8) & 0xFF) as u8, ((k >> 16) & 0xFF) as u8]
        });
        let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
        for px in rgb.chunks_exact(3) {
            rgba.extend_from_slice(px);
            rgba.push(if Some([px[0], px[1], px[2]]) == key { 0 } else { 255 });
        }
        let tex = Texture2D::from_rgba8(w, h, &rgba);
        tex.set_filter(FilterMode::Nearest);
        b.texture = Some(tex.clone());
        Some(tex)
    }

    pub fn volume(&self) -> f32 {
        self.audio.volume
    }

    pub fn set_volume(&mut self, v: f32) {
        self.audio.set_volume(v);
    }

    pub fn muted(&self) -> bool {
        self.audio.muted
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.audio.set_muted(muted);
    }

    /// Is somebody playing on this client?
    pub fn has_local_player(&self) -> bool {
        self.playing.values().any(|&s| s == PlayState::Local)
    }

    pub fn has_sound(&self) -> bool {
        self.audio.enabled
    }

    /// A one-line status report, printed from time to time.
    pub fn stats(&self) -> String {
        format!("{} frames, {} kbytes downloaded, {} sounds loaded, {}{}",
                self.frames_received, self.data.downloaded / 1024,
                self.audio.samples.len(), self.audio.music_status(),
                if self.inline.is_some() { ", frames over TCP" } else { "" })
    }

    // ------------------------------------------------------------------
    // frames and drawing

    fn update_frame(&mut self, frame: Vec<u8>) {
        self.frames_received += 1;
        // sounds come first, as 6-byte records ending with 0xFFFF
        let mut base = 0;
        let mut current = HashMap::new();
        while frame.len() >= base + 6 && frame[base + 4..base + 6] == [0xFF, 0xFF] {
            let key = i16::from_be_bytes([frame[base], frame[base + 1]]) as i64;
            let (l, r) = (frame[base + 2], frame[base + 3]);
            match self.playingsounds.get(&key) {
                Some(&n) if n > 0 => {
                    current.insert(key, n - 1);
                }
                _ => {
                    self.audio.play_sample(key, l as f32 / 255.0, r as f32 / 255.0);
                    current.insert(key, 4);
                }
            }
            base += 6;
        }
        self.playingsounds = current;
        self.frame = frame[base..].to_vec();
    }

    /// Draw the playfield in game coordinates (the caller sets up the
    /// camera / render target).
    pub fn draw(&mut self, time: f64) {
        clear_background(BLACK);
        let frame = std::mem::take(&mut self.frame);
        for s in frame.chunks_exact(6) {
            let x = i16::from_be_bytes([s[0], s[1]]) as f32;
            let y = i16::from_be_bytes([s[2], s[3]]) as f32;
            let code = i16::from_be_bytes([s[4], s[5]]) as i64;
            self.draw_icon(code, x, y);
        }
        self.frame = frame;
        self.animated = false;
        if self.taskbarmode {
            self.draw_taskbar(time);
        }
    }

    fn draw_icon(&mut self, code: i64, x: f32, y: f32) {
        let ico = match self.icons.get(&code) {
            Some(i) => i.clone(),
            None => return,
        };
        if let Some(tex) = self.texture(ico.bmp) {
            draw_texture_ex(&tex, x, y, Color::from_rgba(255, 255, 255, ico.alpha),
                            DrawTextureParams { source: Some(ico.rect), ..Default::default() });
        }
    }

    // ------------------------------------------------------------------
    // the taskbar at the bottom, to join the game and define keys

    fn icon_size(&self, code: i64) -> (f32, f32) {
        self.icons.get(&code).map_or((0.0, 0.0), |i| (i.rect.w, i.rect.h))
    }

    fn nextkeyname(&self) -> Option<String> {
        let (_, df) = self.keydefinition.as_ref()?;
        self.keys.iter()
            .filter(|(name, (_, icons))| !df.contains_key(*name) && !icons.is_empty())
            .min_by_key(|(_, (num, _))| *num)
            .map(|(name, _)| name.clone())
    }

    fn get_taskbar(&mut self, time: f64) -> (f32, Vec<(f32, f32, i64, i64)>) {
        let y0 = self.height - TASKBAR_HEIGHT;
        let mut list = Vec::new();
        let f = (1.5 * time).fract();
        let pi: Vec<(i64, i64)> = self.playericons.iter().map(|(&a, &b)| (a, b)).collect();
        let mut xpos = 0.0;
        for &(id, ico) in &pi {
            let state = *self.playing.get(&id).unwrap_or(&PlayState::Off);
            if state != PlayState::Local {
                let (w, h) = self.icon_size(ico);
                xpos += (w * 5.0 / 3.0).floor();
                if state == PlayState::Off {
                    let mut y = self.height - h;
                    let mut icon = ico;
                    if self.keydefinition.as_ref().map(|k| k.0) == Some(id) {
                        if let Some(name) = self.nextkeyname() {
                            let icons = &self.keys[&name].1;
                            let i = ((f * icons.len() as f64) as isize - 1)
                                .rem_euclid(icons.len() as isize) as usize;
                            icon = icons[i];
                            let (_, kh) = self.icon_size(icon);
                            y = y0 + ((TASKBAR_HEIGHT - kh) / 2.0).floor();
                            self.animated = true;
                        }
                    }
                    list.push((xpos - w, y, icon, id));
                }
            }
        }
        let f2 = f * (1.0 - f) * 4.0;
        let mut xpos = self.width;
        for &(id, ico) in pi.iter().rev() {
            if self.playing.get(&id) == Some(&PlayState::Local) {
                let (w, h) = self.icon_size(ico);
                xpos -= (w * 5.0 / 3.0).floor();
                let dy = TASKBAR_HEIGHT - h - 1.0;
                let y = self.height - h - (dy * f2 as f32).floor();
                list.push((xpos, y, ico, id));
                self.animated = true;
            }
        }
        (y0, list)
    }

    fn draw_taskbar(&mut self, time: f64) {
        let (y0, icons) = self.get_taskbar(time);
        // a blue gradient, like the pygame client
        for j in 0..TASKBAR_HEIGHT as i32 {
            let a = (j as f32 * 256.0 / TASKBAR_HEIGHT) as u8;
            draw_rectangle(0.0, y0 + j as f32, self.width, 1.0, Color::from_rgba(128, 128, 255, a));
        }
        for (x, y, ico, _) in icons {
            self.draw_icon(ico, x, y);
        }
    }

    fn clic_taskbar(&mut self, cx: f32, cy: f32, time: f64) -> Option<i64> {
        let (y0, icons) = self.get_taskbar(time);
        if cy >= y0 {
            for (x, _, ico, id) in icons {
                let (w, _) = self.icon_size(ico);
                if x <= cx && cx < x + w {
                    return Some(id);
                }
            }
        }
        None
    }

    fn settaskbar(&mut self, nmode: bool, time: f64) {
        self.taskbartimeout = None;
        if self.taskbarfree {
            let any_local = self.playing.values().any(|&s| s == PlayState::Local);
            self.taskbarmode = nmode || !any_local || self.keydefinition.is_some();
            if nmode {
                self.taskbartimeout = Some(time + 5.0);
            }
        }
    }

    /// Mouse moved, in game coordinates.
    pub fn pointer_motion(&mut self, _x: f32, y: f32, time: f64) {
        self.settaskbar(y >= self.height - 2.0 * TASKBAR_HEIGHT, time);
    }

    /// Mouse click, in game coordinates.
    pub fn mouse_click(&mut self, x: f32, y: f32, time: f64) {
        self.settaskbar(true, time);
        self.keydefinition = None;
        if let Some(id) = self.clic_taskbar(x, y, time) {
            if self.playing.get(&id) == Some(&PlayState::Local) {
                self.send(message(CMSG_REMOVE_PLAYER, &[id.into()]));
            } else {
                self.keydefinition = Some((id, HashMap::new()));
                self.taskbarmode = true;
            }
        }
    }

    /// A key was pressed (`pressed`) or released.
    pub fn key_event(&mut self, key: KeyCode, pressed: bool) {
        if let Some((pid, msg)) = self.keycodes.get(&(key, pressed)).cloned() {
            if self.playing.get(&pid) == Some(&PlayState::Local) {
                self.send(msg);
                return;
            }
        }
        if pressed && self.keydefinition.is_some() {
            self.define_key(key);
        }
    }

    fn define_key(&mut self, key: KeyCode) {
        let next = match self.nextkeyname() {
            Some(n) => n,
            None => return,
        };
        let (clic_id, df) = self.keydefinition.as_mut().unwrap();
        if df.values().any(|&k| k == key) {
            return;
        }
        df.insert(next, key);
        let clic_id = *clic_id;
        if self.nextkeyname().is_some() {
            return;
        }
        let (_, df) = self.keydefinition.take().unwrap();
        self.send(message(CMSG_ADD_PLAYER, &[clic_id.into()]));
        for (keyname, (num, _)) in &self.keys {
            let (name, pressed) = match keyname.strip_prefix('-') {
                Some(n) => (n, false),
                None => (keyname.as_str(), true),
            };
            if let Some(&k) = df.get(name) {
                self.keycodes.insert((k, pressed),
                                     (clic_id, message(CMSG_KEY, &[clic_id.into(), (*num).into()])));
            }
        }
    }
}
