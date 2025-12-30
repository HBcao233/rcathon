mod ast;
mod repl;

use std::io::{self, Read};
use clap::{CommandFactory, FromArgMatches, Arg, ArgAction};


#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
  /// program passed in as a string (terminates option list)
  #[arg(short = 'c', value_name = "cmd")]
  command: Option<String>,
  
  /// program read from script file
  #[arg(value_name = "file")]
  file: Option<String>,
}


fn main() {
  let cli = Cli::command()
    .version(env!("CARGO_PKG_VERSION"))
    .disable_help_flag(true)
    .disable_version_flag(true)
    .arg(
      Arg::new("version")
        .short('V')
        .long("version")
        .help("Print Version")
        .action(ArgAction::Version)
    )
    .arg(
      Arg::new("help")
        .short('h')
        .short_alias('?')
        .long("help")
        .help("print this help message and exit (also -?)")
        .action(ArgAction::Help)
    );
  
  let matches = cli.get_matches();
  let args = Cli::from_arg_matches(&matches).unwrap();
  
  let mut piped_cmd: Option<String> = None;
  if !atty::is(atty::Stream::Stdin) {
    let mut buffer = String::new();
    let _ = io::stdin().read_to_string(&mut buffer);
    let trimmed = buffer.trim().to_string();
    if !trimmed.is_empty() {
      piped_cmd = Some(trimmed);
    }
  }
  
  if let Some(s) = piped_cmd {
    run_command(&s);
    std::process::exit(0);
  }
  
  if let Some(cmd) = args.file {
    run_command(&cmd);
    std::process::exit(0);
  }
  
  match &args.command {
    Some(file) => {
      run_file(file);
      std::process::exit(0);
    }
    &None => {}
  }
  
  let line = "-1";
  let mut lexer = ast::Lexer::new(line);
  // let tokens = lexer.tokenize_all();
  let mut parser = ast::Parser::new(&mut lexer);
  let node = parser.parse();
  println!("{:?}", node);
  for i in parser.arena.nodes {
    println!("{:?}", i);
  }
  
  // repl::repl();
  
  /*
  let mut parser = match Parser::new(lexer) {
    Ok(p) => p,
    Err(e) => {
      println!("Parse error: {}", e);
    }
  };
  
  match parser.parse() {
    Ok(ast) => match eval(&ast) {
      Ok(v) => println!("{}", v),
      Err(EvalError::DivisionByZero) => println!("Evaluation error: division by zero"),
      Err(EvalError::Other(s)) => println!("Evaluation error: {}", s),
    },
    Err(e) => println!("Parse error: {}", e),
  }
  */
  
  std::process::exit(0);
}


fn run_file(path: &str) {
  // TODO: 实现文件执行逻辑
  println!("正在读取并执行文件: {}", path);
}

fn run_command(cmd: &str) {
  // TODO: 实现命令执行逻辑
  println!("正在执行: {}", cmd);
}