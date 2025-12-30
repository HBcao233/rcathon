use cathon_compiler::CodeObject;
use crate::value::Value;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

/// 调用帧
pub struct Frame {
  /// 代码对象
  pub code: CodeObject,
  /// 指令指针
  pub ip: usize,
  /// 操作数栈
  pub stack: Vec<Value>,
  /// 局部变量
  pub locals: Vec<Value>,
  /// 全局变量引用
  pub globals: Rc<RefCell<HashMap<String, Value>>>,
}

impl Frame {
  pub fn new(code: CodeObject, globals: Rc<RefCell<HashMap<String, Value>>>) -> Self {
    let locals_count = code.varnames.len();
    Self {
      code,
      ip: 0,
      stack: Vec::with_capacity(256),
      locals: vec![Value::None; locals_count],
      globals,
    }
  }

  /// 读取下一个字节
  pub fn read_byte(&mut self) -> u8 {
    let byte = self.code.code[self.ip];
    self.ip += 1;
    byte
  }

  /// 读取双字节参数
  pub fn read_u16(&mut self) -> u16 {
    let high = self.read_byte() as u16;
    let low = self.read_byte() as u16;
    (high << 8) | low
  }

  /// 压栈
  pub fn push(&mut self, value: Value) {
    self.stack.push(value);
  }

  /// 弹栈
  pub fn pop(&mut self) -> Value {
    self.stack.pop().expect("Stack underflow")
  }

  /// 查看栈顶
  pub fn peek(&self) -> &Value {
    self.stack.last().expect("Stack is empty")
  }
}