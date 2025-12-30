use crate::opcode::OpCode;

/// 代码对象 - 类似 CPython 的 PyCodeObject
#[derive(Debug, Clone, PartialEq)]
pub struct CodeObject {
  /// 函数名 (顶层为 "<module>")
  pub name: String,
  /// 字节码
  pub code: Vec<u8>,
  /// 常量池
  pub constants: Vec<Constant>,
  /// 变量名列表 (用于 LOAD_NAME/STORE_NAME)
  pub names: Vec<String>,
  /// 局部变量名 (用于 LOAD_FAST/STORE_FAST)
  pub varnames: Vec<String>,
  /// 参数数量
  pub arg_count: usize,
  /// 行号表 (字节码偏移 -> 源码行号)
  pub line_table: Vec<(usize, usize)>,
}

/// 常量类型
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
  None,
  Bool(bool),
  Int(i64),
  Float(f64),
  String(String),
  /// 嵌套的代码对象 (用于函数定义)
  Code(Box<CodeObject>),
}

impl CodeObject {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      code: Vec::new(),
      constants: Vec::new(),
      names: Vec::new(),
      varnames: Vec::new(),
      arg_count: 0,
      line_table: Vec::new(),
    }
  }

  /// 写入一个字节
  pub fn emit(&mut self, byte: u8) {
    self.code.push(byte);
  }

  /// 写入操作码
  pub fn emit_op(&mut self, op: OpCode) {
    self.emit(op as u8);
  }

  /// 写入操作码 + 操作数
  pub fn emit_op_arg(&mut self, op: OpCode, arg: u16) {
    self.emit(op as u8);
    self.emit((arg >> 8) as u8);   // 高位
    self.emit((arg & 0xFF) as u8); // 低位
  }

  /// 添加常量，返回索引
  pub fn add_const(&mut self, constant: Constant) -> u16 {
    // 检查是否已存在
    if let Some(idx) = self.constants.iter().position(|c| c == &constant) {
        return idx as u16;
    }
    let idx = self.constants.len();
    self.constants.push(constant);
    idx as u16
  }

  /// 添加变量名，返回索引
  pub fn add_name(&mut self, name: String) -> u16 {
    if let Some(idx) = self.names.iter().position(|n| n == &name) {
      return idx as u16;
    }
    let idx = self.names.len();
    self.names.push(name);
    idx as u16
  }

  /// 添加局部变量名
  pub fn add_varname(&mut self, name: String) -> u16 {
    if let Some(idx) = self.varnames.iter().position(|n| n == &name) {
      return idx as u16;
    }
    let idx = self.varnames.len();
    self.varnames.push(name);
    idx as u16
  }

  /// 当前字节码偏移
  pub fn offset(&self) -> usize {
    self.code.len()
  }

  /// 修补跳转地址
  pub fn patch_jump(&mut self, offset: usize) {
    let jump_to = self.code.len() as u16;
    self.code[offset + 1] = (jump_to >> 8) as u8;
    self.code[offset + 2] = (jump_to & 0xFF) as u8;
  }
}