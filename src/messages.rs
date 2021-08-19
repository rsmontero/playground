use base64::decode;
use strum_macros::{Display, EnumString};

use std::convert::AsRef;
use std::io::{BufRead, Cursor};
use std::os::unix::io::RawFd;

use std::str::{from_utf8, FromStr};
use std::string::ToString;

use std::time::SystemTime;

#[derive(Debug)]
pub struct Message<T, const B64: bool>
where
    T: FromStr + ToString,
{
    _type: T,

    _oid: i64,

    _status: String,

    _payload: String,

    _timestamp: SystemTime,
}

impl<T, const B64: bool> ToString for Message<T, B64>
  where
      T: FromStr + ToString,
{
    fn to_string(&self) -> String {
        let mut buffer: String = format!("{} {} {} ",
            self._type.to_string(),
            self._status,
            self._oid);

        if !self._payload.is_empty() {
            buffer.push_str(&self._payload);
        }

        buffer.push('\n');

        buffer
    }
}

impl<T, const B64: bool> Message<T, B64>
where
    T: FromStr + ToString,
{
    pub fn parse<B: AsRef<[u8]>>(buffer: &B) -> Option<Message<T, B64>> {
        let mut cursor = Cursor::new(&buffer);
        let mut token: Vec<u8> = Vec::new();

        let mt = match cursor
            .read_until(b' ', &mut token)
            .ok()
            .and_then(|u: usize| {
                if u <= 1 {
                    None
                } else {
                    from_utf8(&token[..u - 1]).ok()
                }
            })
            .and_then(|s: &str| T::from_str(s).ok())
        {
            Some(s) => s,
            None => return None,
        };

        token.clear();

        let oid = match cursor
            .read_until(b' ', &mut token)
            .ok()
            .and_then(|u: usize| {
                if u <= 1 {
                    None
                } else {
                    from_utf8(&token[..u - 1]).ok()
                }
            })
            .and_then(|s: &str| s.parse::<i64>().ok())
        {
            Some(s) => s,
            None => return None,
        };

        token.clear();

        let st = match cursor
            .read_until(b' ', &mut token)
            .ok()
            .and_then(|u: usize| {
                if u <= 1 {
                    None
                } else {
                    from_utf8(&token[..u - 1]).ok()
                }
            }) {
            Some(s) => s,
            None => return None,
        };

        let mut py: String = String::new();

        if let Err(_) = cursor.read_line(&mut py) {
            return None;
        }

        if B64 {
            py = match decode(py).ok().and_then(|v| String::from_utf8(v).ok()) {
                Some(s) => String::from(s),
                None => return None,
            }
        }

        Some(Message {
            _type: mt,
            _status: String::from(st),
            _payload: py,
            _oid: oid,
            _timestamp: SystemTime::now(),
        })
    }

    pub fn write(&self, fd : RawFd) -> Result<(), &'static str> {
        let s: String = self.to_string();
        let s: &[u8]  = s.as_bytes();

        nix::unistd::write(fd, s)
            .map_err(|e| e.desc())
            .map(|_| ())
    }
}

#[derive(EnumString, Display, Debug)]
pub enum APIMessages {
    Red,
    Blue,
    Yellow,
}

pub type TestMessage = Message<APIMessages, true>;

