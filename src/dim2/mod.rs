#[cfg(feature = "automata")]
pub mod automata;
pub mod chunk;
pub mod grid;

pub type GlobalPos = [i64; 2];
pub type ChunkPos = [i32; 2];
pub type LocalPos = [usize; 2];
