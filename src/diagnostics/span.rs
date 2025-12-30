#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

impl Span {
  pub fn new(start: usize, end: usize) -> Self { 
    Span { start, end }
  }
  
  pub fn len(&self) -> usize {
    self.end.saturating_sub(self.start)
  }
  
  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}