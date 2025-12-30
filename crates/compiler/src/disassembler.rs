use crate::code::CodeObject;
use crate::opcode::OpCode;

pub fn disassemble(code: &CodeObject) {
  println!("=== {} ===", code.name);
  println!("Constants: {:?}", code.constants);
  println!("Names: {:?}", code.names);
  println!();
  
  let mut offset = 0;
  while offset < code.code.len() {
    let op = OpCode::from(code.code[offset]);
    print!("{:04}  {:?}", offset, op);
    
    offset += 1;
    
    // 读取参数 (大部分指令都有2字节参数)
    match op {
        OpCode::LoadConst | OpCode::LoadName | OpCode::StoreName |
        OpCode::LoadFast | OpCode::StoreFast | OpCode::Jump |
        OpCode::JumpIfFalse | OpCode::JumpIfTrue | OpCode::Loop |
        OpCode::Call | OpCode::BuildList => {
            let arg = ((code.code[offset] as u16) << 8) | (code.code[offset + 1] as u16);
            print!(" {}", arg);
            offset += 2;
        }
        _ => {}
    }
    
    println!();
  }
}