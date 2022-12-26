#[macro_use]
mod macros;
#[cfg(feature = "automata")]
pub mod automata;
pub mod chunk;
pub mod grid;

use vecmath::Vector2;

pub use crate::chunk::{Chunk, ChunkSparse};
pub use crate::grid::{ExGrid, ExGridSparse};

pub type GlobalPos = Vector2<i64>;
pub type ChunkPos = Vector2<i32>;
pub type LocalPos = Vector2<usize>;
