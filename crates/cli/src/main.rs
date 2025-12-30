use cathon_core::ast::Lexer;
use cathon_core::ast::Parser;
use cathon_compiler::Compiler;
use cathon_runtime::VM;
use cathon_compiler::disassemble;
// mod repl;


fn main() {
  let source = r#"
print(1 + 2)
"#;

  // 1. 词法分析
  let mut lexer = Lexer::new(source);
  
  // 2. 语法分析
  let mut parser = Parser::new(&mut lexer);
  let module = parser.parse().unwrap();
  let arena = parser.arena;
  
  // 3. 编译为字节码
  let code = Compiler::new().compile(&arena, module).unwrap();
  
  println!("code: {:?}", code);
  disassemble(&code);
  
  // 4. 执行字节码
  let mut vm = VM::new();
  match vm.run(code) {
    Ok(result) => println!("=> {}", result),
    Err(e) => eprintln!("Error: {:?}", e),
  }
  
  // repl::repl();
}
