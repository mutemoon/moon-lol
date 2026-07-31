use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Bincode(#[from] bincode::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}
