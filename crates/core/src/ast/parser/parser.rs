use crate::Span;
use crate::{Error, SyntaxError};
use super::super::TokenKind;
use super::super::Token;
use super::super::TokenStream;
use super::super::Lexer;
use super::nodes::*;

#[derive(Debug)]
pub struct Parser<'a> {
  tokens: TokenStream<'a>,
  pos: usize,
  interner: Interner,
  pub arena: Arena,
}

impl<'a> Parser<'a> {
  pub fn new(lexer: &'a mut Lexer) -> Self {
    Self {
      tokens: lexer.stream(),
      pos: 0,
      interner: Interner::new(),
      arena: Arena::new(),
    }
  }
  
  fn peek(&mut self) -> Option<&Result<Token, Error>> {
    self.tokens.peek(1)
  }

  fn next(&mut self) -> Option<Result<Token, Error>> {
    self.pos += 1;
    self.tokens.next_token()
  }

  fn blanks(&mut self) -> usize {
    let mut count: usize = 0;
    loop {
      match self.peek() {
        Some(Ok(tok)) if tok.kind() == &TokenKind::Newline  => {
          self.next();
          count += 1;
          break;
        },
        Some(_) | None => break,
      }
    }
    count
  }

  pub fn parse(&mut self) -> Result<NodeId, Error> {
    let node = self.file()?;
    // ensure Endmarker
    match self.peek() {
      Some(Ok(tok)) if tok.kind() == &TokenKind::Endmarker => Ok(node),
      Some(_) | None => Err(SyntaxError::new("invalid syntax", Span::new(self.pos, self.pos))),
    }
  }
  
  fn file(&mut self) -> Result<NodeId, Error> {
    let body = self.statements()?;
    let node_first = self.arena.get(body[0]);
    let node_end = self.arena.get(body[0]);
    Ok(self.arena.alloc(
      NodeKind::Module { body }, 
      Span::new(node_first.span().start, node_end.span().end),
    ))
  }
  
  fn statements(&mut self) -> Result<Vec<NodeId>, Error> {
    self.blanks();
    match self.peek() {
      Some(Err(_)) => {
        return Err(self.next().expect("Some").expect_err("Err"));
      },
      Some(Ok(tok)) if tok.kind() == &TokenKind::Endmarker => return Ok(Vec::new()),
      Some(_) => {},
      None => return Err(SyntaxError::new("no eof", Span::new(self.pos, self.pos))),
    }
    let res = self.statement()?;
    /*loop {
      if self.blanks() == 0 {
        break;
      }
    }*/
    Ok(res)
  }
  
  fn statement(&mut self) -> Result<Vec<NodeId>, Error> {
    Ok(self.stmts()?)
  }
  
  fn stmts(&mut self) -> Result<Vec<NodeId>, Error> {
    let mut items = Vec::new();
    let item = self.stmt()?;
    items.push(item);
    Ok(items)
  }
  
  fn stmt(&mut self) -> Result<NodeId, Error> {
    if let Ok(item) = self.compound_stmt() {
      Ok(item)
    } else {
      Ok(self.simple_stmt()?)
    }
  }
  
  fn compound_stmt(&mut self) -> Result<NodeId, Error> {
    Err(SyntaxError::new("not implement", Span::new(self.pos, self.pos)))
  }
  
  fn simple_stmt(&mut self) -> Result<NodeId, Error> {
    let item = self.star_expressions()?;
    Ok(item)
  }
  
  fn star_expressions(&mut self) -> Result<NodeId, Error> {
    let start = self.pos;
    let value = self.star_expression()?;
    match self.peek() {
      Some(Ok(tok)) => {
        let end = tok.span().end;
        Ok(self.arena.alloc(
          NodeKind::Expr { value }, 
          Span::new(start, end),
        ))
      },
      Some(Err(_)) => Err(self.next().expect("Some").expect_err("Err")),
      None => Err(SyntaxError::new("no except", Span::new(self.pos, self.pos))),
    }
  }
  
  fn star_expression(&mut self) -> Result<NodeId, Error> {
    Ok(self.expression()?)
  }
  
  fn expression(&mut self) -> Result<NodeId, Error> {
    Ok(self.disjunction()?)
  }
  
  fn bin_op<F>(
    &mut self, 
    operators: &[TokenKind], 
    func: F,
  ) -> Result<NodeId, Error> 
  where 
    F: Fn(&mut Self) -> Result<NodeId, Error> 
  {
    let mut left = func(self)?;
    
    while let Some(res) = self.peek() {
      match res {
        Ok(tok) if operators.contains(tok.kind()) => {
          let op = self.next().expect("Some").expect("Ok");
          let right = func(self)?;
          left = self.arena.alloc_BinOp(left, op, right);
        },
        Ok(_) => break,
        Err(_) => {
          let err = self.next().expect("Some").expect_err("Err");
          return Err(err);
        },
      }
    }
    Ok(left)
  }
  
  fn disjunction(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::DoubleVBar], Parser::conjunction)?)
  }
  
  fn conjunction(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::DoubleAmper], Parser::inversion)?)
  }
  
  fn inversion(&mut self) -> Result<NodeId, Error> {
    match self.peek() {
      Some(Ok(tok)) if tok.kind() == &TokenKind::Exclamation => Ok(self.bin_op(&[TokenKind::Exclamation], Parser::comparison)?),
      Some(Err(_)) => {
        let err = self.next().expect("Some").expect_err("Err"); 
        Err(err)
      },
      Some(_) | None => Ok(self.comparison()?),
    }
  }
  
  fn comparison(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(
      &[TokenKind::EqEqual, TokenKind::NotEqual, TokenKind::Less, TokenKind::Greater, TokenKind::LessEqual, TokenKind::GreaterEqual], 
      Parser::bitwise_or
    )?)
  }
  
  fn bitwise_or(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::VBar], Parser::bitwise_xor)?)
  }
  
  fn bitwise_xor(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::Circumflex], Parser::bitwise_add)?)
  }
  
  fn bitwise_add(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::Amper], Parser::shift_expr)?)
  }
  
  fn shift_expr(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::LeftShift, TokenKind::RightShift], Parser::sum)?)
  }
  
  fn sum(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::Plus, TokenKind::Minus], Parser::term)?)
  }
  
  fn term(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::Star, TokenKind::Slash, TokenKind::DoubleSlash, TokenKind::Percent, TokenKind::At], Parser::factor)?)
  }
  
  fn factor(&mut self) -> Result<NodeId, Error> {
    let pos_start = self.pos;
    match self.peek() {
      Some(Ok(tok)) if matches!(tok.kind(), TokenKind::Plus | TokenKind::Minus) => {
        let op = self.next().expect("Some").expect("Ok");
        let operand = self.factor()?;
        Ok(self.arena.alloc(
          NodeKind::UnaryOp { 
            op, 
            operand,
          },
          Span::new(pos_start, self.pos),
        ))
      },
      Some(Err(_)) => {
        let err = self.next().expect("Some").expect_err("Err");
        Err(err)
      },
      Some(_) | None => Ok(self.power()?),
    }
  }
  
  fn power(&mut self) -> Result<NodeId, Error> {
    Ok(self.bin_op(&[TokenKind::DoubleStar], Parser::primary)?)
  }
  
  fn primary(&mut self) -> Result<NodeId, Error> {
    Ok(self.atom()?)
  }
  
  fn atom(&mut self) -> Result<NodeId, Error> {
    let start = self.pos;
    match self.next() {
      Some(Ok(tok)) if matches!(tok.kind(), TokenKind::Int(..)) => {
        let span = Span::new(tok.span().start, tok.span().end);
        let node = self.arena.alloc(
          NodeKind::Constant { value: tok },
          span,
        );
        Ok(node)
      },
      Some(Err(_)) => {
        let err = self.next().expect("Some").expect_err("Err");
        Err(err)
      },
      Some(_) | None => Err(SyntaxError::new("invalid atom", Span::new(start, self.pos))),
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_simple() {
    let code = "12 + 2 - 3";
    let mut lexer = Lexer::new(code);
    let mut parser = Parser::new(&mut lexer);
    let node = parser.parse().unwrap();
    assert_eq!(node, 6); 
    let nodes = parser.arena.nodes;

    let expected_texts = [
      "12",
      "2",
      "12 + 2",
      "3",
      "12 + 2 - 3",
      "12 + 2 - 3",
      "12 + 2 - 3",
    ];
    assert_eq!(
      nodes.len(),
      expected_texts.len(),
      "nodes length not match",
    );
    
    for (i, expected) in expected_texts.iter().enumerate() {
      let node = &nodes[i];
      let text = &code[node.span().start..node.span().end];
      assert_eq!(
        &text, 
        expected,
        "node's spaned text not match",
      );
    }
    assert!(matches!(
      nodes[0].kind(),
      &NodeKind::Constant { .. },
    ));
  }
  
  #[test]
  fn parse_error() {
    let code = "1 + ";
    let mut lexer = Lexer::new(code);
    let mut parser = Parser::new(&mut lexer);
    let err = parser.parse().expect_err("no expect");
    let expected = SyntaxError::new("invalid atom", Span::new(2, 3));
    assert_eq!(err.kind(), expected.kind());
    assert_eq!(err.message(), "invalid atom");
    assert_eq!(err.span(), expected.span());
  }
}