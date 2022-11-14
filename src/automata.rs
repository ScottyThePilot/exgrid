use crate::chunk::*;
use crate::grid::*;

use std::collections::hash_map::{Entry, RandomState};
use std::hash::BuildHasher;
use std::mem::swap;



pub trait AutomataRules<const S: usize, H = RandomState> {
  type Cell: Default + Clone;

  /// Rule that determines when new neighboring chunks should be created in order to
  /// prevent the automata from becoming trapped in a limited number of chunks.
  fn expansion(&self, chunk: &Chunk<Self::Cell, S>) -> Expansion8;

  /// Rule that determines the value of a given cell based on the current state of the automata.
  fn simulate(&self, pos: [isize; 2], grid: &ExGrid<Self::Cell, S, H>) -> Self::Cell;

  /// Called once after a step has completed.
  #[allow(unused_variables)]
  fn step_completed(&mut self, grid: &ExGrid<Self::Cell, S, H>) {}

  /// Rule that determines whether or not a cell is "empty".
  /// A chunk containing empty cells will be erased by [`Automata::clean_up`] if all of its cells pass this check.
  /// Defaults to an implementation that returns `false`.
  #[allow(unused_variables)]
  fn empty_cell(&self, cell: &Self::Cell) -> bool {
    false
  }
}

pub struct Automata<A: AutomataRules<S, H>, const S: usize, H = RandomState> {
  automata_rules: A,
  state: ExGrid<A::Cell, S, H>
}

impl<A: AutomataRules<S, H>, H: BuildHasher, const S: usize> Automata<A, S, H> {
  pub fn new(automata_rules: A) -> Self where H: Default {
    Self::with_state(automata_rules, ExGrid::new())
  }

  pub fn with_state(automata_rules: A, state: ExGrid<A::Cell, S, H>) -> Self {
    Automata { automata_rules, state }
  }

  pub fn step(&mut self)
  where A::Cell: Default, H: Default {
    let mut scratch = ExGrid::new();
    self.step_scratch(&mut scratch);
  }

  pub fn step_scratch(&mut self, scratch: &mut ExGrid<A::Cell, S, H>)
  where A::Cell: Default {
    scratch.clear();
    for (&pos, chunk) in self.state.chunks() {
      self.automata_rules.expansion(chunk).apply_with_center(pos, |pos| {
        if let Entry::Vacant(entry) = scratch.get_chunk_entry(pos) {
          let mut chunk = Chunk::<A::Cell, S>::new();
          for (local, value) in chunk.cells_mut() {
            let pos = crate::grid::compose::<S>(pos, local);
            *value = self.automata_rules.simulate(pos, &self.state);
          };

          entry.insert(chunk);
        };
      });
    };

    swap(&mut self.state, scratch);
  }

  pub fn get_rules(&self) -> &A {
    &self.automata_rules
  }

  pub fn get_rules_mut(&mut self) -> &mut A {
    &mut self.automata_rules
  }

  pub fn get_state(&self) -> &ExGrid<A::Cell, S, H> {
    &self.state
  }

  pub fn get_state_mut(&mut self) -> &mut ExGrid<A::Cell, S, H> {
    &mut self.state
  }

  pub fn clean_up(&mut self) {
    self.state.retain(|_, &mut ref chunk| {
      chunk.iter().any(|cell| {
        !self.automata_rules.empty_cell(cell)
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
  pub(crate) fn apply<F>(self, pos: [i32; 2], mut f: F)
  where F: FnMut([i32; 2]) {
    if self.nn { f(add(pos, [0, -1])) };
    if self.ne { f(add(pos, [1, -1])) };
    if self.ee { f(add(pos, [1, 0])) };
    if self.se { f(add(pos, [1, 1])) };
    if self.ss { f(add(pos, [0, 1])) };
    if self.sw { f(add(pos, [-1, 1])) };
    if self.ww { f(add(pos, [-1, 0])) };
    if self.nw { f(add(pos, [-1, -1])) };
  }

  pub(crate) fn apply_with_center<F>(self, pos: [i32; 2], mut f: F)
  where F: FnMut([i32; 2]) {
    f(pos);
    self.apply(pos, f);
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

fn add(a: [i32; 2], b: [i32; 2]) -> [i32; 2] {
  [a[0] + b[0], a[1] + b[1]]
}
