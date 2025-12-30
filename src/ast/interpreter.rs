use super::parser::{BinaryOp, Expr, UnaryOp};

#[derive(Debug)]
pub enum EvalError {
  DivisionByZero,
  Other(String),
}

impl From<String> for EvalError {
  fn from(s: String) -> Self {
    EvalError::Other(s)
  }
}

pub fn eval(expr: &Expr) -> Result<f64, EvalError> {
  match expr {
    Expr::Number(n) => Ok(*n),
    Expr::Unary { op, expr } => {
      let v = eval(expr)?;
      match op {
        UnaryOp::Plus => Ok(v),
        UnaryOp::Minus => Ok(-v),
      }
    }
    Expr::Binary { left, op, right } => {
      let a = eval(left)?;
      let b = eval(right)?;
      match op {
        BinaryOp::Add => Ok(a + b),
        BinaryOp::Sub => Ok(a - b),
        BinaryOp::Mul => Ok(a * b),
        BinaryOp::Div => {
          if b == 0.0 {
            Err(EvalError::DivisionByZero)
          } else {
            Ok(a / b)
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::Lexer;
  use crate::parser::Parser;

  #[test]
  fn eval_expr() {
    let lex = Lexer::new("2*(3+4)-1");
    let mut p = Parser::new(lex).unwrap();
    let ast = p.parse().unwrap();
    let v = eval(&ast).unwrap();
    assert_eq!(v, 13.0);
  }
}