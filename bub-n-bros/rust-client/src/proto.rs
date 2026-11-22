//! Message encoding of the Bub-n-Bros protocol (see common/msgstruct.py).
//!
//! A message is:  n (1 byte) | n bytes of struct type codes | type (1 byte) | values
//! where the values are packed big-endian ("network order") following the
//! type codes, e.g. "llB9s".

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Bytes(Vec<u8>),
}

impl Value {
    pub fn int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }
    pub fn text(&self) -> String {
        match self {
            Value::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
            Value::Int(i) => i.to_string(),
        }
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Value {
        Value::Int(i)
    }
}
impl From<&[u8]> for Value {
    fn from(b: &[u8]) -> Value {
        Value::Bytes(b.to_vec())
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Bytes(s.as_bytes().to_vec())
    }
}

// server -> client
pub const MSG_WELCOME: &[u8] = b"Welcome to gamesrv.py(3) !\n";
pub const MSG_BROADCAST_PORT: u8 = b'*';
pub const MSG_DEF_PLAYFIELD: u8 = b'p';
pub const MSG_DEF_KEY: u8 = b'k';
pub const MSG_DEF_ICON: u8 = b'r';
pub const MSG_DEF_BITMAP: u8 = b'm';
pub const MSG_DEF_SAMPLE: u8 = b'w';
pub const MSG_PLAY_MUSIC: u8 = b'Z';
pub const MSG_FADEOUT: u8 = b'f';
pub const MSG_PLAYER_JOIN: u8 = b'+';
pub const MSG_PLAYER_KILL: u8 = b'-';
pub const MSG_PLAYER_ICON: u8 = b'i';
pub const MSG_PING: u8 = b'g';
pub const MSG_PONG: u8 = b'G';
pub const MSG_INLINE_FRAME: u8 = b'\\';
pub const MSG_PATCH_FILE: u8 = b'z';
pub const MSG_ZPATCH_FILE: u8 = b'P';
pub const MSG_MD5_FILE: u8 = b'M';

// client -> server
pub const CMSG_PROTO_VERSION: u8 = b'v';
pub const CMSG_KEY: u8 = b'k';
pub const CMSG_ADD_PLAYER: u8 = b'+';
pub const CMSG_REMOVE_PLAYER: u8 = b'-';
pub const CMSG_UDP_PORT: u8 = b'<';
pub const CMSG_ENABLE_SOUND: u8 = b's';
pub const CMSG_ENABLE_MUSIC: u8 = b'm';
pub const CMSG_PING: u8 = b'g';
pub const CMSG_PONG: u8 = b'G';
pub const CMSG_DATA_REQUEST: u8 = b'M';
pub const CMSG_PLAYER_NAME: u8 = b'n';

/// Encode a message, choosing the type codes like msgstruct.message().
pub fn message(tp: u8, values: &[Value]) -> Vec<u8> {
    let mut codes = String::new();
    let mut body = Vec::new();
    for v in values {
        match v {
            Value::Bytes(b) => {
                codes.push_str(&format!("{}s", b.len()));
                body.extend_from_slice(b);
            }
            Value::Int(i) if (0..256).contains(i) => {
                codes.push('B');
                body.push(*i as u8);
            }
            Value::Int(i) => {
                codes.push('l');
                body.extend_from_slice(&(*i as i32).to_be_bytes());
            }
        }
    }
    let mut out = Vec::with_capacity(2 + codes.len() + body.len());
    out.push(codes.len() as u8);
    out.extend_from_slice(codes.as_bytes());
    out.push(tp);
    out.extend_from_slice(&body);
    out
}

/// Size in bytes of one struct item.
fn item_size(code: u8) -> Option<usize> {
    Some(match code {
        b'B' | b'b' | b'c' | b'x' | b'?' => 1,
        b'h' | b'H' => 2,
        b'l' | b'L' | b'i' | b'I' | b'f' => 4,
        b'q' | b'Q' | b'd' => 8,
        _ => return None,
    })
}

pub enum Decoded {
    /// a complete message: (type, values, bytes consumed)
    Message(u8, Vec<Value>, usize),
    /// not enough data yet
    Incomplete,
    /// garbage
    Error,
}

/// Decode one message at the start of `data`.
pub fn decode(data: &[u8]) -> Decoded {
    if data.is_empty() {
        return Decoded::Incomplete;
    }
    let n = data[0] as usize;
    if data.len() < n + 2 {
        return Decoded::Incomplete;
    }
    let codes = &data[1..1 + n];
    let tp = data[1 + n];
    let mut pos = 2 + n;
    let mut values = Vec::new();
    let mut i = 0;
    while i < codes.len() {
        let mut count = 0usize;
        let mut has_count = false;
        while i < codes.len() && codes[i].is_ascii_digit() {
            count = count * 10 + (codes[i] - b'0') as usize;
            has_count = true;
            i += 1;
        }
        if i >= codes.len() {
            return Decoded::Error;
        }
        let code = codes[i];
        i += 1;
        if !has_count {
            count = 1;
        }
        if code == b's' {
            if data.len() < pos + count {
                return Decoded::Incomplete;
            }
            values.push(Value::Bytes(data[pos..pos + count].to_vec()));
            pos += count;
            continue;
        }
        let size = match item_size(code) {
            Some(s) => s,
            None => return Decoded::Error,
        };
        for _ in 0..count {
            if data.len() < pos + size {
                if pos + size > 1_000_000 {
                    return Decoded::Error;
                }
                return Decoded::Incomplete;
            }
            let b = &data[pos..pos + size];
            let v = match code {
                b'B' | b'c' | b'?' => b[0] as i64,
                b'b' => b[0] as i8 as i64,
                b'h' => i16::from_be_bytes([b[0], b[1]]) as i64,
                b'H' => u16::from_be_bytes([b[0], b[1]]) as i64,
                b'l' | b'i' => i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64,
                b'L' | b'I' => u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64,
                b'q' => i64::from_be_bytes(b.try_into().unwrap()),
                b'Q' => u64::from_be_bytes(b.try_into().unwrap()) as i64,
                b'x' => {
                    pos += size;
                    continue;
                }
                _ => 0,
            };
            pos += size;
            if code == b'c' {
                values.push(Value::Bytes(vec![v as u8]));
            } else {
                values.push(Value::Int(v));
            }
        }
    }
    Decoded::Message(tp, values, pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_matches_python() {
        // python: message(MSG_DEF_PLAYFIELD, 640, 480, 0, 'Bub & Bob', -5, True)
        let m = message(
            MSG_DEF_PLAYFIELD,
            &[640.into(), 480.into(), 0.into(), "Bub & Bob".into(), (-5).into(), 1.into()],
        );
        let expected: &[u8] = b"\x07llB9slBp\x00\x00\x02\x80\x00\x00\x01\xe0\x00Bub & Bob\xff\xff\xff\xfb\x01";
        assert_eq!(m, expected);
        match decode(&m) {
            Decoded::Message(tp, v, used) => {
                assert_eq!(tp, MSG_DEF_PLAYFIELD);
                assert_eq!(used, m.len());
                assert_eq!(v[0], Value::Int(640));
                assert_eq!(v[3], Value::Bytes(b"Bub & Bob".to_vec()));
                assert_eq!(v[4], Value::Int(-5));
            }
            _ => panic!("decode failed"),
        }
        assert!(matches!(decode(&m[..m.len() - 1]), Decoded::Incomplete));
    }
}
