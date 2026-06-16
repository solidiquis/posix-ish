use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

pub trait ToLrsResult<T> {
    fn invalid_argument<S: Into<String>>(self, msg: S) -> Result<T>;
}

#[derive(Debug)]
pub enum Error {
    InvalidArgument(String),
    Unreachable,
}

impl Error {
    pub fn invalid_argument<S: Into<String>>(msg: S) -> Self {
        Self::InvalidArgument(msg.into())
    }

    pub fn unreachable() -> Self {
        Self::Unreachable
    }
}

impl<T> From<Error> for Result<T> {
    fn from(err: Error) -> Self {
        Err(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! write_err {
            ($category:expr, $ctx:expr, $inner:expr) => {
                write!(
                    f,
                    "error: {}\n\n\
                    Caused by:\n  \
                    - {}\n  \
                    - {}
                    ",
                    $category,
                    $ctx,
                    $inner,
                )
            };
            ($category:expr, $ctx:expr) => {
                write!(
                    f,
                    "error: {}\n\n\
                    Caused by:\n  \
                    - {}
                    ",
                    $category,
                    $ctx
                )
            };
        }
        match self {
            Self::InvalidArgument(msg) => {
                write_err!("InvalidArgument", msg)
            }
            Self::Unreachable => {
                write_err!("Unreachable", "this error shouldn't happen")
            }
        }
    }
}

impl<T, E> ToLrsResult<T> for std::result::Result<T, E> {
    fn invalid_argument<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(_err) => Err(Error::InvalidArgument(msg.into()))
        }
    }
}
