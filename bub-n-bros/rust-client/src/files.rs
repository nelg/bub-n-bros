//! Game data files (images, sounds, music) sent by the server.
//!
//! The server describes each piece of a file with MSG_MD5_FILE (file id,
//! path, offset, length, MD5).  We look for a matching piece in our local
//! cache directory; only if it is missing do we ask the server for it
//! (CMSG_DATA_REQUEST), and we store what we receive in the cache so that
//! the next game starts faster.  This mirrors display/caching.py.

use md5::{Digest, Md5};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Default)]
pub struct DataFile {
    /// position -> data
    pub blocks: BTreeMap<usize, Vec<u8>>,
    /// positions requested from the server and not received yet
    pub pending: Vec<usize>,
    /// cache file name, if known
    pub cachename: Option<PathBuf>,
}

impl DataFile {
    pub fn ready(&self) -> bool {
        self.pending.is_empty() && !self.blocks.is_empty()
    }

    /// The data received so far, with holes filled with zeroes.
    pub fn read(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for (&pos, data) in &self.blocks {
            if result.len() < pos + data.len() {
                result.resize(pos + data.len(), 0);
            }
            result[pos..pos + data.len()].copy_from_slice(data);
        }
        result
    }

    /// Have bytes start..end all been received?
    pub fn covers(&self, start: usize, end: usize) -> bool {
        let mut covered = start;
        for (&pos, data) in self.blocks.range(..end) {
            let bend = pos + data.len();
            if bend <= covered {
                continue;
            }
            if pos > covered {
                return false;
            }
            covered = bend;
            if covered >= end {
                return true;
            }
        }
        covered >= end
    }

    /// Bytes start..end, if they have all been received.
    pub fn slice(&self, start: usize, end: usize) -> Option<Vec<u8>> {
        let mut out = vec![0u8; end - start];
        let mut covered = start;
        for (&pos, data) in self.blocks.range(..end) {
            let bend = pos + data.len();
            if bend <= start {
                continue;
            }
            if pos > covered {
                return None; // hole
            }
            let a = pos.max(start);
            let b = bend.min(end);
            if b > a {
                out[a - start..b - start].copy_from_slice(&data[a - pos..b - pos]);
                covered = covered.max(b);
            }
        }
        if covered >= end { Some(out) } else { None }
    }

    /// Length of the data available without holes from position 0.
    pub fn contiguous_len(&self) -> usize {
        let mut end = 0;
        for (&pos, data) in &self.blocks {
            if pos > end {
                break;
            }
            end = end.max(pos + data.len());
        }
        end
    }
}

pub struct DataStore {
    pub files: HashMap<i64, DataFile>,
    cachedir: Option<PathBuf>,
    pub downloaded: usize,
}

fn mangle(filename: &str) -> String {
    filename
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' { c } else { '_' })
        .collect()
}

pub fn default_cache_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("BUBNBROS_CACHE") {
        return Some(PathBuf::from(d));
    }
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("bubnbros").join("cache"))
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|d| PathBuf::from(d).join("Library/Caches/bubnbros"))
    } else if let Some(d) = std::env::var_os("XDG_CACHE_HOME") {
        Some(PathBuf::from(d).join("bubnbros"))
    } else {
        std::env::var_os("HOME").map(|d| PathBuf::from(d).join(".cache").join("bubnbros"))
    }
}

impl DataStore {
    pub fn new(cachedir: Option<PathBuf>) -> DataStore {
        DataStore { files: HashMap::new(), cachedir, downloaded: 0 }
    }

    /// MSG_MD5_FILE: returns true if the data must be requested from the server.
    pub fn md5_file(&mut self, fileid: i64, filename: &str, position: usize,
                    length: usize, checksum: &[u8]) -> bool {
        let cachename = self.cachedir.as_ref().map(|d| d.join(mangle(filename)));
        let file = self.files.entry(fileid).or_default();
        file.cachename = cachename.clone();
        // already received (e.g. requested by the music prefetcher)?
        if let Some(data) = file.slice(position, position + length) {
            if Md5::digest(&data).as_slice() == checksum {
                return false;
            }
        }
        if file.pending.contains(&position) {
            return false; // already asked for
        }
        if let Some(name) = &cachename {
            if let Ok(mut f) = fs::File::open(name) {
                let mut data = vec![0u8; length];
                if f.seek(SeekFrom::Start(position as u64)).is_ok()
                    && f.read_exact(&mut data).is_ok()
                    && Md5::digest(&data).as_slice() == checksum
                {
                    file.blocks.insert(position, data);
                    return false;
                }
            }
        }
        if !file.pending.contains(&position) {
            file.pending.push(position);
        }
        true
    }

    /// MSG_PATCH_FILE / MSG_ZPATCH_FILE: data from the server.
    pub fn patch(&mut self, fileid: i64, position: usize, data: Vec<u8>, transferred: usize) {
        self.downloaded += transferred;
        let file = self.files.entry(fileid).or_default();
        if let Some(name) = &file.cachename {
            let _ = (|| -> std::io::Result<()> {
                if let Some(parent) = name.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut f = OpenOptions::new().create(true).write(true).truncate(false).open(name)?;
                f.seek(SeekFrom::Start(position as u64))?;
                f.write_all(&data)
            })();
        }
        file.pending.retain(|&p| p != position);
        file.blocks.insert(position, data);
    }
}
