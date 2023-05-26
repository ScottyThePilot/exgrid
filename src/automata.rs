use crate::{GlobalPos, ChunkPos};
use crate::chunk::*;
use crate::grid::*;
use crate::vector::Vector2;

use num_traits::Signed;

use std::collections::hash_map::Entry;
use std::hash::BuildHasher;
use std::mem::swap;



/// Describes a generic structure upon which a cellular automata may operate.
pub trait AutomataAdapter: Sized {
  type Cell;
  type CellPos: Copy;

  type Chunk;
  type ChunkPos: Copy;

  fn step_scratch(&mut self, scratch: &mut Self, automata: &mut impl Automata<Self>);

  fn step(&mut self, automata: &mut impl Automata<Self>) where Self: Default {
    self.step_scratch(&mut Self::default(), automata);
  }
}

impl<T, const S: usize, H> AutomataAdapter for ExGrid<T, S, H>
where T: Default, H: BuildHasher {
  type Cell = T;
  type CellPos = GlobalPos;

  type Chunk = Chunk<T, S>;
  type ChunkPos = ChunkPos;

  fn step_scratch(&mut self, scratch: &mut Self, automata: &mut impl Automata<Self>) {
    scratch.clear();
    for (&chunk_pos, chunk) in self.chunks() {
      automata.expansion(chunk).apply_with_center(chunk_pos, |chunk_pos| {
        if let Entry::Vacant(entry) = scratch.get_chunk_entry(chunk_pos) {
          let mut chunk = Chunk::new();
          for (local, value) in chunk.cells_mut() {
            let pos = crate::grid::compose::<S>(chunk_pos, local);
            *value = automata.simulate(pos, &self);
          };

          entry.insert(chunk);
        };
      });
    };

    swap(self, scratch);
  }
}

impl<T, const S: usize, H> AutomataAdapter for ExGridSparse<T, S, H>
where H: BuildHasher {
  type Cell = Option<T>;
  type CellPos = GlobalPos;

  type Chunk = ChunkSparse<T, S>;
  type ChunkPos = ChunkPos;

  fn step_scratch(&mut self, scratch: &mut Self, automata: &mut impl Automata<Self>) {
    scratch.clear();
    for (&chunk_pos, chunk) in self.chunks() {
      automata.expansion(chunk).apply_with_center(chunk_pos, |chunk_pos| {
        if let Entry::Vacant(entry) = scratch.get_chunk_entry(chunk_pos) {
          let mut chunk = Chunk::new();
          for (local, value) in chunk.cells_mut() {
            let pos = crate::grid::compose::<S>(chunk_pos, local);
            *value = automata.simulate(pos, &self);
          };

          entry.insert(chunk.into());
        };
      });
    };

    swap(self, scratch);
  }
}

/// The rules and external state of a celluar automata.
pub trait Automata<A: AutomataAdapter> {
  type Expansion: Expansion<A::ChunkPos>;

  /// Rule that determines when new neighboring chunks should be created in order to
  /// prevent the automata from becoming trapped in a limited number of chunks.
  fn expansion(&mut self, chunk: &A::Chunk) -> Self::Expansion;

  /// Rule that determines the value of a given cell based on the current state of the automata.
  fn simulate(&mut self, pos: A::CellPos, grid: &A) -> A::Cell;
}

pub trait Expansion<P: Copy>: Sized {
  fn apply(self, pos: P, function: impl FnMut(P));

  fn apply_with_center(self, pos: P, mut function: impl FnMut(P)) {
    function(pos);
    self.apply(pos, function);
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expansion4 {
  /// Vector `[ 0, -1]`
  pub n: bool,
  /// Vector `[ 0,  1]`
  pub s: bool,
  /// Vector `[ 1,  0]`
  pub e: bool,
  /// Vector `[-1,  0]`
  pub w: bool
}

impl Expansion4 {
  pub(crate) fn rel<T>() -> [Vector2<T>; 4] where T: Signed {
    [
      Vector2 { x: T::zero(), y: -T::one() },
      Vector2 { x: T::zero(), y: T::one() },
      Vector2 { x: T::one(), y: T::zero() },
      Vector2 { x: -T::one(), y: T::zero() }
    ]
  }
}

impl<T> Expansion<[T; 2]> for Expansion4 where T: Signed + Copy {
  fn apply(self, pos: [T; 2], mut f: impl FnMut([T; 2])) {
    let pos = Vector2::from_array(pos);
    let rel = Self::rel();
    if self.n { f(Vector2::into_array(pos + rel[0])) };
    if self.s { f(Vector2::into_array(pos + rel[1])) };
    if self.e { f(Vector2::into_array(pos + rel[2])) };
    if self.w { f(Vector2::into_array(pos + rel[3])) };
  }
}

impl Default for Expansion4 {
  fn default() -> Self {
    Expansion4 {
      n: false,
      s: false,
      e: false,
      w: false
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expansion8 {
  /// Vector `[ 0, -1]`
  pub nn: bool,
  /// Vector `[ 1, -1]`
  pub ne: bool,
  /// Vector `[ 1,  0]`
  pub ee: bool,
  /// Vector `[ 1,  1]`
  pub se: bool,
  /// Vector `[ 0,  1]`
  pub ss: bool,
  /// Vector `[-1,  1]`
  pub sw: bool,
  /// Vector `[-1,  0]`
  pub ww: bool,
  /// Vector `[-1, -1]`
  pub nw: bool
}

impl Expansion8 {
  pub(crate) fn rel<T>() -> [Vector2<T>; 8] where T: Signed {
    [
      Vector2 { x: T::zero(), y: -T::one() },
      Vector2 { x: T::one(), y: -T::one() },
      Vector2 { x: T::one(), y: T::zero() },
      Vector2 { x: T::one(), y: T::one() },
      Vector2 { x: T::zero(), y: T::one() },
      Vector2 { x: -T::one(), y: T::one() },
      Vector2 { x: -T::one(), y: T::zero() },
      Vector2 { x: -T::one(), y: -T::one() }
    ]
  }
}

impl<T> Expansion<[T; 2]> for Expansion8 where T: Signed + Copy {
  fn apply(self, pos: [T; 2], mut f: impl FnMut([T; 2])) {
    let pos = Vector2::from_array(pos);
    let rel = Self::rel();
    if self.nn { f(Vector2::into_array(pos + rel[0])) };
    if self.ne { f(Vector2::into_array(pos + rel[1])) };
    if self.ee { f(Vector2::into_array(pos + rel[2])) };
    if self.se { f(Vector2::into_array(pos + rel[3])) };
    if self.ss { f(Vector2::into_array(pos + rel[4])) };
    if self.sw { f(Vector2::into_array(pos + rel[5])) };
    if self.ww { f(Vector2::into_array(pos + rel[6])) };
    if self.nw { f(Vector2::into_array(pos + rel[7])) };
  }
}

impl From<Expansion4> for Expansion8 {
  fn from(expansion4: Expansion4) -> Self {
    Expansion8 {
      nn: expansion4.n,
      ss: expansion4.s,
      ee: expansion4.e,
      ww: expansion4.w,
      ne: expansion4.n && expansion4.e,
      se: expansion4.s && expansion4.e,
      sw: expansion4.s && expansion4.w,
      nw: expansion4.n && expansion4.w
    }
  }
}

impl Default for Expansion8 {
  fn default() -> Self {
    Expansion8 {
      nn: false,
      ne: false,
      ee: false,
      se: false,
      ss: false,
      sw: false,
      ww: false,
      nw: false
    }
  }
}

impl<T, const S: usize> ChunkSparse<T, S> {
  /// This function checks the edges of a given chunk, if any cells on a given edge
  /// fails to pass the provided predicate or it is empty, that edge is marked.
  pub fn edges_not_empty_expansion(&self) -> Expansion4 {
    let s = S - 1;
    Expansion4 {
      n: self.horizontal_slice_iter(0).any(Option::is_some),
      s: self.horizontal_slice_iter(s).any(Option::is_some),
      w: self.vertical_slice_iter(0).any(Option::is_some),
      e: self.vertical_slice_iter(s).any(Option::is_some),
    }
  }
}
