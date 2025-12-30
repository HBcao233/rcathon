use cathon::Error;
use super::super::TokenKind;
use super::super::Token;
use super::Lexer;
use std::collections::VecDeque;

/// TokenStream 在 Lexer（或任何产生 Token 的迭代器）外包装一层缓冲，支持任意 n 的 peek（peek(1) 是下一个 token）
#[derive(Debug)]
pub struct TokenStream<'a> {
  iter: &'a mut Lexer,
  buf: VecDeque<Result<Token, Error>>,
}

impl<'a> TokenStream<'a> {
  pub fn new(iter: &'a mut Lexer) -> Self {
    Self {
      iter,
      buf: VecDeque::new(),
    }
  }

  /// 确保缓冲区至少有 n 个元素（如果遇到 EOF 会停止填充）
  fn ensure_buffered(&mut self, n: usize) {
    while self.buf.len() < n && let Some(x) = self.iter.next() {
      match x {
        Ok(tok) if tok.kind() == &TokenKind::Endmarker => {
          self.buf.push_back(Ok(tok));
          break;
        },
        Ok(_) => self.buf.push_back(x),
        Err(_) => {
          self.buf.push_back(x);
          break
        },
      }
    }
  }

  /// peek 第 n 个未消费 token（n 从 1 开始）
  pub fn peek(&mut self, n: usize) -> Option<&Result<Token, Error>> {
    if n == 0 {
      return None;
    }
    self.ensure_buffered(n);
    self.buf.get(n - 1)
  }

  /// 取出并消费下一个 token（owned）
  pub fn next_token(&mut self) -> Option<Result<Token, Error>> {
    if let Some(t) = self.buf.pop_front() {
      Some(t)
    } else {
      self.iter.next()
    }
  }
}

impl<'a> Iterator for TokenStream<'a> {
  type Item = Result<Token, Error>;
  fn next(&mut self) -> Option<Self::Item> {
    self.next_token()
  }
}