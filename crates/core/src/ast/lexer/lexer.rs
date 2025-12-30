use std::collections::VecDeque;
use crate::Span;
use crate::{Error, SyntaxError, TabError, IndentationError};
use super::super::TokenKind;
use super::super::Token;
use super::TokenStream;

#[derive(Debug, PartialEq)]
enum IndentType {
  Space,
  Tab,
}

#[derive(Debug)]
pub struct Lexer {
  chars: Vec<char>,
  pos: usize,
  eof_emitted: bool,
  first_token: bool,
  indent_type: Option<IndentType>,
  indents: Vec<usize>,
  buffer: VecDeque<Token>,
}

impl Lexer {
  pub fn new(input: &str) -> Self {
    Self { 
      chars: input.chars().collect(), 
      pos: 0,
      eof_emitted: false,
      first_token: true,
      indent_type: None,
      indents: Vec::new(),
      buffer: VecDeque::new(),
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
    loop {
      match (&mut self).next() {
        Some(Ok(tok)) if tok.kind() == &TokenKind::Endmarker => {
          tokens.push(tok);
          break;
        },
        Some(Ok(tok)) => tokens.push(tok),
        Some(Err(err)) => return Err(err),
        None => return Err(SyntaxError::new("no expected end", Span::new(self.pos, self.pos))),
      }
    }
    Ok(tokens)
  }
  
  pub fn stream(&mut self) -> TokenStream<'_> {
    TokenStream::new(self)
  }
  
  fn next_token(&mut self) -> Option<Result<Token, Error>> {
    if self.pos >= self.chars.len() {
      return self.emit_eof();
    }
    if self.buffer.len() > 0 {
      return Some(Ok(self.buffer.pop_front().expect("has buffer")));
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
    // 换行与缩进
    if c == '\n' {
      self.advance();
      return match self.make_indent() {
        Err(err) => Some(Err(err)),
        Ok(mut tokens) => {
          if !self.first_token {
            tokens.push_front(Token::new(
              TokenKind::Newline,
              Span::new(self.pos, self.pos),
            ));
          };
          match tokens.pop_front() {
            Some(_) if self.first_token => Some(Err(IndentationError::new("unexpected indent", Span::new(self.pos, self.pos)))),
            Some(tok) => {
              for i in tokens {
                self.buffer.push_back(i);
              }
              Some(Ok(tok))
            },
            None => self.next_token(),
          }
        }
      }
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
      if c == ' ' {
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
  
  fn make_indent(&mut self) -> Result<VecDeque<Token>, Error> {
    let start = self.pos;
    while let Some(d) = self.peek_char() {
      if d == ' ' || d == '\t' {
        let indent_type = if d == ' ' { IndentType::Space } else { IndentType::Tab };
        match &self.indent_type {
          Some(x) if x != &indent_type => return Err(TabError::new("inconsistent use of tabs and spaces in indentation", Span::new(self.pos, self.pos))),
          Some(_) => {},
          None => {self.indent_type = Some(indent_type)},
        } 
        self.advance();
      } else {
        break;
      }
    }
    let end = self.pos;
    let count: usize = end - start;
    
    let mut res = VecDeque::new();
    while self.indents.len() > 0 {
      if &count >= self.indents.last().expect("len>0") {
        break
      }
      res.push_back(Token::new(
        TokenKind::Dedent(self.indents.len()),
        Span::new(end, end),
      ));
      self.indents.pop();
    }
    
    if count > 0 {
      // 是否为新的缩进
      /* let mut new_indent = self.indents.iter().filter(|x| x == count).count() == 0;
      if self.peek_char() == '\n' {
        new_indent = false;
      }*/
      let last = self.indents.last();
      if last == None || &count > last.expect("Not None") {
        self.indents.push(count);
        res.push_back(Token::new(
          TokenKind::Indent(self.indents.len()),
          Span::new(end, end),
        ));
      }
    }
    
    Ok(res)
  }
}

impl Iterator for &mut Lexer {
  type Item = Result<Token, Error>;
  
  fn next(&mut self) -> Option<Self::Item> {
    let token = self.next_token();
    if let Some(_) = token {
      self.first_token = false;
    }
    token
  }
}


#[cfg(test)]
mod tests {
  use super::Lexer;
  use super::TokenKind;
  use crate::errors::Error;
  use crate::errors::ErrorKind;
  
  // 测试：数字与运算符（注意：不要使用浮点文字 like "3.5" ——当前词法器尚未支持小数点，会触发 todo!()）
  #[test]
  fn numbers_and_ops() {
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
  
  // 测试：空输入只产生 Endmarker
  #[test]
  fn empty_input() {
    let toks = Lexer::new("").tokenize_all().unwrap();
    // 仅一个 Endmarker
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind(), &TokenKind::Endmarker);
  }
  
  // 测试：空白跳过与简单数字
  #[test]
  fn tab_error() {
    let s = "   \n\t 42 ";
    let err = Lexer::new(s).tokenize_all().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Tab);
  }
  
  // 测试：确保只发出一个 EOF（Endmarker）
  #[test]
  fn eof_emitted_once() {
    let s = "1";
    let toks = Lexer::new(s).tokenize_all().unwrap();
    let eof_count = toks.iter().filter(|t| *t.kind() == TokenKind::Endmarker).count();
    assert_eq!(eof_count, 1, "Endmarker 应该只出现一次");
  }

  // 测试缩进
  #[test]
  fn indent() {
    let s = r#"
1 
  2
    3 
    3 
  2

1
    "#;
    
    let expected = [
      TokenKind::Int(1),
      TokenKind::Newline,
      TokenKind::Indent(1),
      TokenKind::Int(2),
      TokenKind::Newline,
      TokenKind::Indent(2),
      TokenKind::Int(3),
      TokenKind::Newline,
      TokenKind::Int(3),
      TokenKind::Newline,
      TokenKind::Dedent(2),
      TokenKind::Int(2),
      TokenKind::Newline,
      TokenKind::Dedent(1),
      TokenKind::Newline,
      TokenKind::Int(1),
      TokenKind::Newline,
      TokenKind::Endmarker,
    ];
    let tokens = Lexer::new(s).tokenize_all().unwrap();
    
    for (i, token) in tokens.iter().enumerate() {
      assert_eq!(token.kind(), &expected[i]);
    }
  }
}