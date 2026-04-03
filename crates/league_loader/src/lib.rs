pub mod game;
pub mod map;
pub mod prop_bin;
pub mod reader;
pub mod wad;
pub mod wad_parse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Nom(String),

    #[error("{0}")]
    Custom(&'static str),
}
