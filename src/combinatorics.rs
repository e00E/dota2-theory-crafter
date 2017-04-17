use std::iter::{Iterator, FlatMap};
use std::vec::Vec;

pub struct Combinations<T: Clone> {
  pool: Vec<T>,
  count: usize,
  state: Vec<usize>,
}

pub struct CombinationsWithReplacement<T: Clone> {
  pool: Vec<T>,
  count: usize,
  state: Vec<usize>,
}

pub fn combination_range<InputIterator, OutputIterator, OutputIteratorCreator>
  (inputs: InputIterator,
   output_iterator_creator: OutputIteratorCreator)
   -> FlatMap<InputIterator, OutputIterator, OutputIteratorCreator>
  where InputIterator: Iterator,
        OutputIterator: Iterator,
        OutputIteratorCreator: Fn(InputIterator::Item) -> OutputIterator
{
  inputs.flat_map(output_iterator_creator)
}

impl<T: Clone> Combinations<T> {
  pub fn new(pool: Vec<T>, count: usize) -> Combinations<T> {
    assert!(pool.len() >= count);
    Combinations {
      pool: pool,
      count: count,
      state: Vec::with_capacity(count),
    }
  }
}

impl<T: Clone> CombinationsWithReplacement<T> {
  pub fn new(pool: Vec<T>, count: usize) -> CombinationsWithReplacement<T> {
    assert!(if count > 0 { pool.len() > 0 } else { true });
    CombinationsWithReplacement {
      pool: pool,
      count: count,
      state: Vec::with_capacity(count),
    }
  }
}

impl<T: Clone> Iterator for Combinations<T> {
  type Item = Vec<T>;
  fn next(&mut self) -> Option<Vec<T>> {
    if self.count == 0 {
      return None;
    } //Can this be done at a better place?
    let mut combination = Vec::with_capacity(self.count);
    if self.state.len() == 0 {
      // This is the first call to next, set up state
      for i in (0..self.count) {
        self.state.push(i);
      }
    } else {
      // Go to the next state if possible and return it, if not we are done
      let mut i = self.count - 1;
      loop {
        if self.state[i] != i + self.pool.len() - self.count {
          break;
        } else if i == 0 {
          return None;
        }
        // This was the last combination
        else {
          i -= 1;
        }
      }
      self.state[i] += 1;
      for j in (i + 1..self.count) {
        self.state[j] = self.state[j - 1] + 1;
      }
    }
    for &i in self.state.iter() {
      combination.push(self.pool[i].clone());
    }
    Some(combination)
  }
}

impl<T: Clone> Iterator for CombinationsWithReplacement<T> {
  type Item = Vec<T>;
  fn next(&mut self) -> Option<Vec<T>> {
    if self.count == 0 {
      return None;
    } //Can this be done at a better place?
    let mut combination = Vec::with_capacity(self.count);
    if self.state.len() == 0 {
      // This is the first call to next, set up state
      for _ in (0..self.count) {
        self.state.push(0);
      }
    } else {
      // Go to the next state if possible and return it, if not we are done
      let mut i = self.count - 1;
      loop {
        if self.state[i] != self.pool.len() - 1 {
          break;
        } else if i == 0 {
          return None;
        }
        // This was the last combination
        else {
          i -= 1;
        }
      }
      let next = self.state[i] + 1;
      for j in (i..self.count) {
        self.state[j] = next;
      }
    }
    for &i in self.state.iter() {
      combination.push(self.pool[i].clone());
    }
    Some(combination)
  }
}
