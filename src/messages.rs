use base64::decode;

use std::convert::AsRef;
use std::io::{BufRead, Cursor};
use std::os::unix::io::RawFd;

use std::str::{from_utf8, FromStr};
use std::string::ToString;

pub trait Message {
    type MessageType;

    fn message_type(&self) -> &Self::MessageType;

    fn parse<B: AsRef<[u8]>>(buffer: &B) -> Option<Self>
    where
        Self: Sized;

    fn write(&self, fd: RawFd) -> Result<(), &'static str>;
}

#[derive(Debug)]
pub struct DriverMessage<T, const B64: bool, const TSTP: bool>
where
    T: FromStr + ToString,
{
    mtype: T,

    oid: i64,

    status: String,

    payload: String,

    timestamp: i64,
}

impl<T, const B64: bool, const TSTP: bool> ToString for DriverMessage<T, B64, TSTP>
where
    T: FromStr + ToString,
{
    fn to_string(&self) -> String {
        let mut buffer: String =
            format!("{} {} {} ", self.mtype.to_string(), self.status, self.oid);

        if !self.payload.is_empty() {
            buffer.push_str(&self.payload);
        }

        buffer.push('\n');

        buffer
    }
}

//impl<T: FromStr + ToString> Message for DriverMessage<T> {
impl<T, const B64: bool, const TSTP: bool> Message for DriverMessage<T, B64, TSTP>
where
    T: FromStr + ToString,
{
    type MessageType = T;

    fn message_type(&self) -> &Self::MessageType {
        &self.mtype
    }

    fn parse<B: AsRef<[u8]>>(buffer: &B) -> Option<Self>
    where
        Self: Sized,
    {
        let mut cursor = Cursor::new(&buffer);
        let mut token: Vec<u8> = Vec::new();

        let mt: T = match DriverMessage::<T, B64, TSTP>::read_word(&mut cursor, &mut token)
            .and_then(|s: &str| T::from_str(s).ok())
        {
            Some(s) => s,
            None => return None,
        };

        token.clear();

        let oid: i64 = match DriverMessage::<T, B64, TSTP>::read_word(&mut cursor, &mut token)
            .and_then(|s: &str| s.parse::<i64>().ok())
        {
            Some(s) => s,
            None => return None,
        };

        token.clear();

        let tstp = if TSTP {
            match DriverMessage::<T, B64, TSTP>::read_word(&mut cursor, &mut token)
                .and_then(|s: &str| s.parse::<i64>().ok())
            {
                Some(s) => {
                    token.clear();
                    s
                }
                None => return None,
            }
        } else {
            0
        };

        let st = match DriverMessage::<T, B64, TSTP>::read_word(&mut cursor, &mut token) {
            Some(s) => String::from(s),
            None => return None,
        };

        let mut py: String = String::new();

        if let Err(_) = cursor.read_line(&mut py) {
            return None;
        }

        if B64 {
            py = match decode(py.trim())
                .ok()
                .and_then(|v| String::from_utf8(v).ok())
            {
                Some(s) => String::from(s),
                None => return None,
            }
        }

        Some(DriverMessage {
            mtype: mt,
            status: st,
            payload: py,
            oid: oid,
            timestamp: tstp,
        })
    }

    fn write(&self, fd: RawFd) -> Result<(), &'static str> {
        let s: String = self.to_string();
        let s: &[u8] = s.as_bytes();

        nix::unistd::write(fd, s).map_err(|e| e.desc()).map(|_| ())
    }
}

impl<T, const B64: bool, const TSTP: bool> DriverMessage<T, B64, TSTP>
where
    T: FromStr + ToString,
{
    fn read_word<'a, B>(cursor: &mut Cursor<B>, token: &'a mut Vec<u8>) -> Option<&'a str>
    where
        B: AsRef<[u8]>,
    {
        loop {
            match cursor.read_until(b' ', token) {
                Ok(u) => match u {
                    0 => return None, //End Of Cursor
                    1 => (),          //' '
                    _ => return from_utf8(&token[..u - 1]).ok(),
                },
                Err(_) => return None,
            };

            token.clear();
        }
    }
}
