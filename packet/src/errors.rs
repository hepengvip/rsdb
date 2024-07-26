use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IOErr;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum PacketError {
    IOError(IOErr),
    FromUtf8Error(FromUtf8Error),
}

impl Error for PacketError {}

impl From<IOErr> for PacketError {
    fn from(e: IOErr) -> Self {
        PacketError::IOError(e)
    }
}

impl From<FromUtf8Error> for PacketError {
    fn from(e: FromUtf8Error) -> Self {
        PacketError::FromUtf8Error(e)
    }
}

impl Display for PacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(e) => {
                write!(f, "{e}")
            }
            Self::FromUtf8Error(e) => {
                write!(f, "{e}")
            }
        }
    }
}

pub type PacketResult<T> = Result<T, PacketError>;
