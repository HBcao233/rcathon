use cathon_core::ast::{Arena, NodeId, NodeKind, TokenKind};
use crate::code::{CodeObject, Constant};
use crate::opcode::OpCode;

pub struct Compiler<'a> {
  arena: Option<&'a Arena>,
  /// 代码对象栈 (用于嵌套函数)
  code_stack: Vec<CodeObject>,
}

impl<'a> Compiler<'a> {
  pub fn new() -> Self {
    Self {
      arena: None,
      code_stack: vec![CodeObject::new("<module>")],
    }
  }

  /// 当前代码对象
  fn code(&mut self) -> &mut CodeObject {
    self.code_stack.last_mut().unwrap()
  }

  /// 编译整个程序
  pub fn compile(mut self, arena: &'a Arena, module: NodeId) -> Result<CodeObject, CompileError> {
    self.arena = Some(arena);
    let node = arena.get(module);
    match node.kind() {
      NodeKind::Module { body } => {
        for node_id in body {
          self.compile_stmt(*node_id)?;
        }
      },
      _ => panic!("need module"),
    }
    
    // 确保返回 None
    self.emit_op(OpCode::LoadConst);
    let idx = self.code().add_const(Constant::None);
    self.emit_arg(idx);
    self.emit_op(OpCode::Return);
    
    Ok(self.code_stack.pop().unwrap())
  }

  /// 编译语句
  fn compile_stmt(&mut self, node_id: NodeId) -> Result<(), CompileError> {
    let arena = self.arena.expect("a");
    let node = arena.get(node_id);
    match node.kind() {
      // 表达式语句
      NodeKind::Expr {value} => {
        self.compile_expr(*value)?;
        self.emit_op(OpCode::Pop);
      },
      _ => todo!(),
    }
    Ok(())
  }

  /// 编译表达式
  fn compile_expr(&mut self, expr_id: NodeId) -> Result<(), CompileError> {
    let arena = self.arena.expect("a");
    let node = arena.get(expr_id);
    match node.kind() {
      NodeKind::Constant {value} => {
        let idx = match value.kind() {
          TokenKind::Int(v) => self.code().add_const(Constant::Int(*v)),
          _ => todo!(),
        };
        
        self.emit_op(OpCode::LoadConst);
        self.emit_arg(idx);
      }

      NodeKind::BinOp { left, op, right } => {
        self.compile_expr(*left)?;
        self.compile_expr(*right)?;
        
        let opcode = match op.kind() {
          TokenKind::Plus => OpCode::BinaryAdd,
          TokenKind::Minus => OpCode::BinarySub,
          /*BinOp::Mul => OpCode::BinaryMul,
          BinOp::Div => OpCode::BinaryDiv,
          BinOp::FloorDiv => OpCode::BinaryFloorDiv,
          BinOp::Mod => OpCode::BinaryMod,
          BinOp::Pow => OpCode::BinaryPow,
          BinOp::Eq => OpCode::CompareEq,
          BinOp::Ne => OpCode::CompareNe,
          BinOp::Lt => OpCode::CompareLt,
          BinOp::Le => OpCode::CompareLe,
          BinOp::Gt => OpCode::CompareGt,
          BinOp::Ge => OpCode::CompareGe,*/
          _ => todo!(),
        };
        self.emit_op(opcode);
      },
      
      _ => todo!(),
    }
    Ok(())
  }

  fn emit_op(&mut self, op: OpCode) {
    self.code().emit_op(op);
  }

  fn emit_arg(&mut self, arg: u16) {
    self.code().emit((arg >> 8) as u8);
    self.code().emit((arg & 0xFF) as u8);
  }
}

#[derive(Debug)]
pub struct CompileError {
  pub message: String,
}
