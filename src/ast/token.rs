use cathon::Span;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenKind {
  Endmarker,
  Name(String),
  Int(i64),
  Float(f64),
  String(String),
  Newline,
  Indent,
  Dedent,
  LPar,
  RPar,
  LSqb,
  RSqb,
  Colon,
  Comma,
  Semi,
  Plus,
  Minus,
  Star,
  Slash,
  VBar,  // |
  Amper,  // &
  Less,
  Greater,
  Equal,
  Dot,
  Percent,
  LBrace,
  RBrace,
  EqEqual,
  NotEqual,
  LessEqual,
  GreaterEqual,
  DoubleVBar,  // &&
  DoubleAmper,  // ||
  Tilde,
  Circumflex,  // ^
  LeftShift,
  RightShift,
  DoubleStar,
  PlusEqual,
  MinEqual,
  StarEqual,
  SlashEqual,
  PercentEqual,
  AmperEqual,
  VBarEqual,
  CircumflexEqual,
  LeftShiftEqual,
  RightShiftEqual,
  DoubleStarEqual,
  DoubleSlash,
  DoubleSlashEqual,
  At,
  AtEqual,
  RArrow,
  Ellipsis,
  ColonEqual,
  Exclamation,
  OP,
  TypeIgnore,
  TypeComment,
  SoftKeyword,
  FStringStart,
  FStringMiddle,
  FStringEnd,
  TStringStart,
  TStringMiddle,
  TStringEnd,
  Comment,
  Nl,
  ErrorToken,
  Encoding,
  NTokens,
}

#[derive(Debug, Clone)]
pub struct Token {
  kind: TokenKind,
  span: Span,
}

impl Token {
  pub fn new(kind: TokenKind, span: Span) -> Self {
    Self { kind, span }
  }
  
  pub fn kind(&self) -> &TokenKind {
    &self.kind
  }
  
  pub fn span(&self) -> Span {
    self.span
  }
}

impl Token {
  pub fn eof(pos: usize) -> Self {
    Self {
      kind: TokenKind::Endmarker,
      span: Span::new(pos, pos),
    }
  }
}