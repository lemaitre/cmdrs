/*
SSH_FXP_STATUS: 101
| u32: id | u32: status code | u32: error length | u8[error length]: error | u32: language length | u8[language length]: language |
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::{Error, Message};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
pub enum StatusCode {
    Ok = 0,
    Eof = 1,
    NoSuchFile = 2,
    PermissionDenied = 3,
    Failure = 4,
    BadMessage = 5,
    NoConnection = 6,
    ConnectionLost = 7,
    OpUnsupported = 8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Status {
    pub code: u32,
    pub error: Bytes,
    pub language: Bytes,
}

impl SftpDecode for Status {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let code = u32::decode(buf)?;
        let error = Bytes::decode(buf)?;
        let language = Bytes::decode(buf)?;
        Ok(Status {
            code,
            error,
            language,
        })
    }
}

impl SftpEncode for &Status {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.code.encode(buf)?;
        self.error.encode(buf)?;
        self.language.encode(buf)?;

        Ok(())
    }
}

impl StatusCode {
    pub fn to_status(self, msg: Bytes) -> Status {
        let msg = if msg.is_empty() {
            self.to_string().into()
        } else {
            msg
        };

        Status {
            code: self as u32,
            error: msg,
            language: "en".into(),
        }
    }

    pub fn to_message(self, msg: Bytes) -> Message {
        Message::Status(self.to_status(msg))
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StatusCode::Ok => "Ok",
            StatusCode::Eof => "EOF",
            StatusCode::NoSuchFile => "No such file",
            StatusCode::PermissionDenied => "Permission Denied",
            StatusCode::Failure => "Failure",
            StatusCode::BadMessage => "Bad message",
            StatusCode::NoConnection => "No connection",
            StatusCode::ConnectionLost => "Connection lost",
            StatusCode::OpUnsupported => "Operation not supported",
        };
        f.write_str(s)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.error.is_empty() {
            write!(f, "{}", self.code)
        } else {
            write!(
                f,
                "{}: {}",
                self.code,
                String::from_utf8_lossy(self.error.as_ref())
            )
        }
    }
}

impl TryFrom<u32> for StatusCode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == Self::Ok as u32 {
            Ok(Self::Ok)
        } else if value == Self::Eof as u32 {
            Ok(Self::Eof)
        } else if value == Self::NoSuchFile as u32 {
            Ok(Self::NoSuchFile)
        } else if value == Self::PermissionDenied as u32 {
            Ok(Self::PermissionDenied)
        } else if value == Self::Failure as u32 {
            Ok(Self::Failure)
        } else if value == Self::BadMessage as u32 {
            Ok(Self::BadMessage)
        } else if value == Self::NoConnection as u32 {
            Ok(Self::NoConnection)
        } else if value == Self::ConnectionLost as u32 {
            Ok(Self::ConnectionLost)
        } else if value == Self::OpUnsupported as u32 {
            Ok(Self::OpUnsupported)
        } else {
            Err(value)
        }
    }
}

impl From<std::io::ErrorKind> for StatusCode {
    fn from(value: std::io::ErrorKind) -> Self {
        match value {
            std::io::ErrorKind::NotFound => Self::NoSuchFile,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::ConnectionRefused => Self::NoConnection,
            std::io::ErrorKind::ConnectionReset => Self::ConnectionLost,
            std::io::ErrorKind::ConnectionAborted => Self::ConnectionLost,
            std::io::ErrorKind::NotConnected => Self::NoConnection,
            std::io::ErrorKind::InvalidInput => Self::BadMessage,
            std::io::ErrorKind::InvalidData => Self::BadMessage,
            std::io::ErrorKind::Unsupported => Self::OpUnsupported,
            std::io::ErrorKind::UnexpectedEof => Self::Eof,
            _ => Self::Failure,
        }
    }
}

impl From<std::io::Error> for Status {
    fn from(value: std::io::Error) -> Self {
        Self {
            code: StatusCode::from(value.kind()) as u32,
            error: value.to_string().into(),
            language: "en".into(),
        }
    }
}

impl From<russh::Error> for Status {
    fn from(value: russh::Error) -> Self {
        let status_code = match value {
            russh::Error::ChannelOpenFailure(_) => StatusCode::NoConnection,
            russh::Error::Disconnect => StatusCode::ConnectionLost,
            russh::Error::IO(io) => {
                return io.into();
            }
            _ => StatusCode::Failure,
        };
        Self {
            code: status_code as u32,
            error: value.to_string().into(),
            language: "en".into(),
        }
    }
}

impl From<Status> for std::io::Error {
    fn from(value: Status) -> Self {
        let kind = match StatusCode::try_from(value.code) {
            Ok(StatusCode::Ok) => std::io::ErrorKind::Other,
            Ok(StatusCode::Eof) => std::io::ErrorKind::UnexpectedEof,
            Ok(StatusCode::NoSuchFile) => std::io::ErrorKind::NotFound,
            Ok(StatusCode::PermissionDenied) => std::io::ErrorKind::PermissionDenied,
            Ok(StatusCode::Failure) => std::io::ErrorKind::Other,
            Ok(StatusCode::BadMessage) => std::io::ErrorKind::InvalidData,
            Ok(StatusCode::NoConnection) => std::io::ErrorKind::Other,
            Ok(StatusCode::ConnectionLost) => std::io::ErrorKind::Other,
            Ok(StatusCode::OpUnsupported) => std::io::ErrorKind::Unsupported,
            Err(_) => std::io::ErrorKind::Other,
        };

        Self::new(kind, value)
    }
}

impl std::error::Error for Status {}
