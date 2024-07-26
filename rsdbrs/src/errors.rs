use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IOErr;
use std::string::FromUtf8Error;

extern crate packet;
use packet::PacketError;

#[derive(Debug)]
pub enum RsDBError {
    IOError(IOErr),
    RespError(String),
    FromUtf8Error(FromUtf8Error),
    NotConnect,
    NoDbSelected,
    EmptyToken,
    PacketError(PacketError),
}

impl Error for RsDBError {}

impl From<IOErr> for RsDBError {
    fn from(e: IOErr) -> Self {
        RsDBError::IOError(e)
    }
}

impl From<FromUtf8Error> for RsDBError {
    fn from(e: FromUtf8Error) -> Self {
        RsDBError::FromUtf8Error(e)
    }
}

impl From<PacketError> for RsDBError {
    fn from(e: PacketError) -> Self {
        Self::PacketError(e)
    }
}

impl Display for RsDBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(e) => {
                write!(f, "{e}")
            }
            Self::RespError(ref msg) => {
                write!(f, "RespError - {msg}")
            }
            Self::NotConnect => {
                write!(f, "Not connect to server")
            }
            Self::NoDbSelected => {
                write!(f, "No database selected")
            }
            Self::EmptyToken => {
                write!(f, "Should not using a empty token")
            }
            Self::FromUtf8Error(e) => {
                write!(f, "{e}")
            }
            Self::PacketError(e) => {
                write!(f, "{e}")
            }
        }
    }
}

pub type RsDBResult<T> = Result<T, RsDBError>;
