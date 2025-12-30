use crate::ast::{Lexer, Parser};
use std::io::{self, Write};

pub fn repl<'a>() {
  println!("Rust Calculator (type 'quit' or 'exit' to leave)");
  loop {
    print!("> ");
    io::stdout().flush().unwrap();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
      println!("Failed to read input");
      continue;
    }
    let line = line.trim();
    if line.is_empty() {
      println!();
      continue;
    }
    if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") {
      break;
    }

    let mut lexer = Lexer::new(line);
    println!("{:?}", lexer);
    let mut parser = Parser::new(&mut lexer);

    match parser.parse() {
      Ok(ast) => {
        println!("{:?}", ast);
        /*Ok(v) => println!("{}", v),
        Err(EvalError::DivisionByZero) => println!("Evaluation error: division by zero"),
        Err(EvalError::Other(s)) => println!("Evaluation error: {}", s),*/
      },
      Err(e) => println!("Parse error: {}", e),
    }
  }
  println!("Goodbye!");
}