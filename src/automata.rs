use crate::{GlobalPos, ChunkPos};
use crate::chunk::*;
use crate::grid::*;

use std::collections::hash_map::{Entry, RandomState};
use std::hash::BuildHasher;
use std::mem::swap;



pub trait AutomataRules<const S: usize> {
  type Cell: Default + Clone;

  /// Rule that determines when new neighboring chunks should be created in order to
  /// prevent the automata from becoming trapped in a limited number of chunks.
  fn expansion(&mut self, chunk: &Chunk<Self::Cell, S>) -> Expansion8;

  /// Rule that determines the value of a given cell based on the current state of the automata.
  fn simulate<H>(&mut self, pos: GlobalPos, grid: &ExGrid<Self::Cell, S, H>) -> Self::Cell;

  /// Rule that determines whether or not a cell is "empty".
  /// A chunk containing empty cells will be erased by [`Automata::clean_up`] if all of its cells pass this check.
  /// Defaults to an implementation that returns `false`.
  #[allow(unused_variables)]
  fn empty_cell(&self, cell: &Self::Cell) -> bool {
    false
  }
}

pub struct Automata<A: AutomataRules<S>, const S: usize, H = RandomState> {
  pub state: ExGrid<A::Cell, S, H>,
  pub rules: A
}

impl<A: AutomataRules<S>, H: BuildHasher, const S: usize> Automata<A, S, H> {
  pub fn new(rules: A) -> Self where H: Default {
    Self::with_state(rules, ExGrid::new())
  }

  pub fn with_state(rules: A, state: ExGrid<A::Cell, S, H>) -> Self {
    Automata { rules, state }
  }

  /// Advances the cellular automata forward by one step.
  pub fn step(&mut self)
  where A::Cell: Default, H: Default {
    let mut scratch = ExGrid::new();
    self.step_scratch(&mut scratch);
  }

  /// Advances the cellular automata forward by one step.
  /// A reference to a grid must be provided for use as scratch state.
  /// The previous state will be written into the scratch state.
  pub fn step_scratch(&mut self, scratch: &mut ExGrid<A::Cell, S, H>)
  where A::Cell: Default {
    scratch.clear();
    for (&pos, chunk) in self.state.chunks() {
      self.rules.expansion(chunk).apply_with_center(pos, |pos| {
        if let Entry::Vacant(entry) = scratch.get_chunk_entry(pos) {
          let mut chunk = Chunk::<A::Cell, S>::new();
          for (local, value) in chunk.cells_mut() {
            let pos = crate::grid::compose::<S>(pos, local);
            *value = self.rules.simulate(pos, &self.state);
          };

          entry.insert(chunk);
        };
      });
    };

    swap(&mut self.state, scratch);
  }

  pub fn clean_up(&mut self) {
    self.state.retain(|_, &mut ref chunk| {
      chunk.iter().any(|cell| {
        !self.rules.empty_cell(cell)
      })
    });
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expansion4 {
  /// Vector [0, -1]
  pub north: bool,
  /// Vector [0, 1]
  pub south: bool,
  /// Vector [1, 0]
  pub east: bool,
  /// Vector [-1, 0]
  pub west: bool
}

impl Default for Expansion4 {
  fn default() -> Self {
    Expansion4 {
      north: false,
      south: false,
      east: false,
      west: false
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expansion8 {
  pub nn: bool, // [0, -1]
  pub ne: bool, // [1, -1]
  pub ee: bool, // [1, 0]
  pub se: bool, // [1, 1]
  pub ss: bool, // [0, 1]
  pub sw: bool, // [-1, 1]
  pub ww: bool, // [-1, 0]
  pub nw: bool  // [-1, -1]
}

impl Expansion8 {
  pub(crate) fn apply<F>(self, pos: ChunkPos, mut f: F)
  where F: FnMut(ChunkPos) {
    if self.nn { f(pos + [0, -1]) };
    if self.ne { f(pos + [1, -1]) };
    if self.ee { f(pos + [1, 0]) };
    if self.se { f(pos + [1, 1]) };
    if self.ss { f(pos + [0, 1]) };
    if self.sw { f(pos + [-1, 1]) };
    if self.ww { f(pos + [-1, 0]) };
    if self.nw { f(pos + [-1, -1]) };
  }

  pub(crate) fn apply_with_center<F>(self, pos: ChunkPos, mut f: F)
  where F: FnMut(ChunkPos) {
    f(pos);
    self.apply(pos, f);
  }
}

impl From<Expansion4> for Expansion8 {
  fn from(expansion4: Expansion4) -> Self {
    Expansion8 {
      nn: expansion4.north,
      ss: expansion4.south,
      ee: expansion4.east,
      ww: expansion4.west,
      ne: expansion4.north && expansion4.east,
      se: expansion4.south && expansion4.east,
      sw: expansion4.south && expansion4.west,
      nw: expansion4.north && expansion4.west
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

/// This function checks the edges of a given chunk, if any cells on a given edge
/// fail to pass the provided rules' `empty_cell` check, that edge is marked.
/// I.e. if the top edge of the provided chunk has one cell that fails the `empty_cell` check,
/// then `nn` (north) will be set to `true` on the returned `Expansion8`.
pub fn edges_not_empty_expansion<A, const S: usize>(rules: &A, chunk: &Chunk<A::Cell, S>) -> Expansion8
where A: AutomataRules<S> {
  let s = S - 1;
  let pred = |cell: &A::Cell| !rules.empty_cell(cell);
  Expansion8::from(Expansion4 {
    north: chunk.horizontal_slice_iter(0).any(pred),
    south: chunk.horizontal_slice_iter(s).any(pred),
    west: chunk.vertical_slice_iter(0).any(pred),
    east: chunk.vertical_slice_iter(s).any(pred),
  })
}
