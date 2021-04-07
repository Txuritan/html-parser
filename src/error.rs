#[derive(Debug)]
pub enum Error {
    Parsing(String),
    Cli(String),
    IO(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parsing(err) => err.fmt(f),
            Error::Cli(err) => err.fmt(f),
            Error::IO(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parsing(_) => None,
            Error::Cli(_) => None,
            Error::IO(err) => Some(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
