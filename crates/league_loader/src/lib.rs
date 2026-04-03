mod game;
mod map;
mod prop_bin;
mod reader;
mod wad;
mod wad_parse;

pub use game::*;
pub use map::*;
pub use prop_bin::*;
pub use reader::*;
use thiserror::Error;
pub use wad::*;
pub use wad_parse::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Nom(String),

    #[error("{0}")]
    Custom(&'static str),
}
