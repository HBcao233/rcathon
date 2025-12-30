use super::Span;
use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
  Syntax,
  Indentation,
  Runtime,
  Tab,
}

#[derive(Debug)]
pub struct Error {
  kind: ErrorKind,
  message: String,
  span: Span,
}

impl Error {
  fn new<M: Into<String>>(kind: ErrorKind, message: M, span: Span) -> Self {
    Self {
      kind,
      message: message.into(),
      span,
    }
  }

  /// Return the error kind (analogous to `io::Error::kind()`).
  pub fn kind(&self) -> ErrorKind {
    self.kind
  }

  /// Return the short human-readable message.
  pub fn message(&self) -> &str {
    &self.message
  }

  /// Return the span where the error applies.
  pub fn span(&self) -> &Span {
    &self.span
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let kind_name = match self.kind {
      ErrorKind::Syntax => "SyntaxError",
      ErrorKind::Indentation => "IndentationError",
      ErrorKind::Runtime => "RuntimeError",
      ErrorKind::Tab => "TabError",
    };
    write!(f, "{}: {} at {}..{}", kind_name, self.message, self.span.start, self.span.end)
  }
}

impl std::error::Error for Error {}


pub struct SyntaxError;
pub struct IndentationError;
pub struct RuntimeError;
pub struct TabError;

impl SyntaxError {
  pub fn new<M: Into<String>>(message: M, span: Span) -> Error {
    Error::new(ErrorKind::Syntax, message, span)
  }
}

impl TabError {
  pub fn new<M: Into<String>>(message: M, span: Span) -> Error {
    Error::new(ErrorKind::Tab, message, span)
  }
}

impl IndentationError {
  pub fn new<M: Into<String>>(message: M, span: Span) -> Error {
    Error::new(ErrorKind::Indentation, message, span)
  }
}

impl RuntimeError {
  pub fn new<M: Into<String>>(message: M, span: Span) -> Error {
    Error::new(ErrorKind::Runtime, message, span)
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn create_and_inspect_errors() {
    let s = Span::new(1, 4);
    let se = SyntaxError::new("unexpected token", s.clone());
    assert_eq!(se.message(), "unexpected token");
    assert_eq!(se.span(), &s);
    assert_eq!(se.kind(), ErrorKind::Syntax);
  }
  
  #[test]
  fn other_error_kinds() {
    let s = Span::new(0, 0);
    let ie = IndentationError::new("bad indent", s.clone());
    assert_eq!(ie.kind(), ErrorKind::Indentation);
  
    let re = RuntimeError::new("divide by zero", s.clone());
    assert_eq!(re.kind(), ErrorKind::Runtime);
  
    let te = TabError::new("tab found", s.clone());
    assert_eq!(te.kind(), ErrorKind::Tab);
  }
}