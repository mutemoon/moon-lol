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
pub use wad::*;
pub use wad_parse::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Binrw(#[from] binrw::Error),

    #[error("{0}")]
    Custom(&'static str),
}
