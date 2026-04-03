mod game;
mod map;
pub mod prop_bin;
mod reader;
mod wad;
mod wad_parse;

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
