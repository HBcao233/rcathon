use cathon::Span;
use cathon::{Error, SyntaxError};
use super::super::TokenKind;
use super::super::Token;
use super::TokenStream;

#[derive(Debug)]
pub struct Lexer {
  chars: Vec<char>,
  pos: usize,
  eof_emitted: bool,
}

impl Lexer {
  pub fn new(input: &str) -> Self {
    Self { 
      chars: input.chars().collect(), 
      pos: 0,
      eof_emitted: false,
    }
  }
  
  pub fn peek_char(&self) -> Option<char> {
    self.chars.get(self.pos).copied()
  } 

  pub fn advance(&mut self) -> Option<char> {
    if self.pos < self.chars.len() {
      let c = self.chars[self.pos];
      self.pos += 1;
      Some(c)
    } else {
      None
    }
  }

  // convenience to get all tokens (useful for debugging/tests)
  pub fn tokenize_all(mut self) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut stream = self.stream();
    loop {
      match stream.next_token() {
        Some(Ok(tok)) if tok.kind() == &TokenKind::Endmarker => {
          tokens.push(tok);
          break;
        },
        Some(Ok(tok)) => tokens.push(tok),
        Some(Err(err)) => return Err(err),
        None => return Err(SyntaxError::new("no except end", Span::new(self.pos, self.pos))),
      }
    }
    Ok(tokens)
  }
  
  pub fn stream(&mut self) -> TokenStream<'_> {
    TokenStream::new(self)
  }
  
  /// 如果已经遍历结束，则先发一个 EOF（只发一次），之后返回 None
  fn emit_eof(&mut self) -> Option<Result<Token, Error>> {
    if self.eof_emitted {
      return None;
    } else {
      self.eof_emitted = true;
      return Some(Ok(Token::eof(self.pos)));
    }
  }
  
  fn skip_whitespace(&mut self) {
    while let Some(c) = self.peek_char() {
      if c.is_whitespace() {
        self.advance();
      } else {
        break;
      }
    }
  }
  
  fn make_number(&mut self) -> Option<Result<Token, Error>> {
    let start = self.pos;
    while let Some(d) = self.peek_char() {
      if d.is_ascii_digit() {
        self.advance();
      } else {
        break;
      }
    }
    let end = self.pos;
    let text: String = self.chars[start..end].iter().collect();
    let value = text.parse::<i64>().unwrap_or(0);
    Some(Ok(Token::new(
      TokenKind::Int(value),
      Span::new(start, end),
    )))
  }
}

impl Iterator for &mut Lexer {
  type Item = Result<Token, Error>;
  
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.chars.len() {
      return self.emit_eof();
    }

    // 跳过空白
    self.skip_whitespace();

    let c = match self.peek_char() {
      Some(ch) => ch,
      // 走到这里说明刚好到尾（处理上面同 EOF），返回 EOF
      None => return self.emit_eof(),
    };

    // 数字
    if c.is_ascii_digit() {
      return self.make_number();
    }

    // 单字符 token
    let start = self.pos;
    self.advance(); // consume c
    let end = self.pos;
    let kind = match c {
      '+' => TokenKind::Plus,
      '-' => TokenKind::Minus,
      '*' => TokenKind::Star,
      '/' => TokenKind::Slash,
      _ => return Some(Err(SyntaxError::new("invalid syntax", Span::new(start, end)))),
    };
    Some(Ok(Token::new( 
      kind,
      Span::new(start, end)
    )))
  }
}


#[cfg(test)]
mod tests {
  use super::Lexer;
  use super::TokenKind;
  
  // 测试：数字与运算符（注意：不要使用浮点文字 like "3.5" ——当前词法器尚未支持小数点，会触发 todo!()）
  #[test]
  fn lex_numbers_and_ops() {
    let s = "12 + 2 - 3";
    let toks = Lexer::new(s).tokenize_all().expect("no except");
  
    // 最后一个 token 应该是 Endmarker（tokenize_all 保证包含 Endmarker）
    assert!(!toks.is_empty());
    assert_eq!(toks.last().unwrap().kind(), &TokenKind::Endmarker);
  
    // 核心 token（去掉末尾 Endmarker）
    let core = &toks[..toks.len()-1];
  
    let expected_texts = vec!["12", "+", "2", "-", "3"];
    let actual_texts: Vec<&str> = core.iter().map(|t| &s[t.span().start..t.span().end]).collect();
    assert_eq!(actual_texts, expected_texts);
  
    // 第一个 token 要是整数 12
    assert!(matches!(core[0].kind(), TokenKind::Int(x) if *x == 12));
  }
  
  // 测试：空白跳过与简单数字
  #[test]
  fn lex_whitespace_and_number() {
    let s = "   \n\t 42 ";
    let toks = Lexer::new(s).tokenize_all().unwrap();
    assert!(matches!(toks[0].kind(), TokenKind::Int(x) if *x == 42));
    assert_eq!(toks.last().unwrap().kind(), &TokenKind::Endmarker);
  }
  
  // 测试：空输入只产生 Endmarker
  #[test]
  fn lex_empty_input() {
    let toks = Lexer::new("").tokenize_all().unwrap();
    // 仅一个 Endmarker
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind(), &TokenKind::Endmarker);
  }
  
  // 测试：确保只发出一个 EOF（Endmarker）
  #[test]
  fn lex_eof_emitted_once() {
    let s = "1";
    let toks = Lexer::new(s).tokenize_all().unwrap();
    let eof_count = toks.iter().filter(|t| *t.kind() == TokenKind::Endmarker).count();
    assert_eq!(eof_count, 1, "Endmarker 应该只出现一次");
  }

}