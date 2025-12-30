use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use cathon_compiler::CodeObject;

/// 运行时值
#[derive(Debug, Clone)]
pub enum Value {
  None,
  Bool(bool),
  Int(i64),
  Float(f64),
  String(Rc<String>),
  List(Rc<RefCell<Vec<Value>>>),
  Dict(Rc<RefCell<HashMap<String, Value>>>),
  Function(Rc<Function>),
  NativeFunction(NativeFn),
}

/// 函数对象
#[derive(Debug)]
pub struct Function {
  pub code: CodeObject,
  pub globals: Rc<RefCell<HashMap<String, Value>>>,
}

/// 原生函数
#[derive(Clone)]
pub struct NativeFn {
  pub name: String,
  pub func: fn(Vec<Value>) -> Result<Value, String>,
}

impl std::fmt::Debug for NativeFn {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "<builtin function {}>", self.name)
  }
}

impl Value {
  pub fn is_truthy(&self) -> bool {
    match self {
      Value::None => false,
      Value::Bool(b) => *b,
      Value::Int(n) => *n != 0,
      Value::Float(f) => *f != 0.0,
      Value::String(s) => !s.is_empty(),
      Value::List(list) => !list.borrow().is_empty(),
      _ => true,
    }
  }

  pub fn type_name(&self) -> &'static str {
    match self {
      Value::None => "NoneType",
      Value::Bool(_) => "bool",
      Value::Int(_) => "int",
      Value::Float(_) => "float",
      Value::String(_) => "str",
      Value::List(_) => "list",
      Value::Dict(_) => "dict",
      Value::Function(_) => "function",
      Value::NativeFunction(_) => "builtin_function",
    }
  }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::None => write!(f, "None"),
      Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
      Value::Int(n) => write!(f, "{}", n),
      Value::Float(n) => write!(f, "{}", n),
      Value::String(s) => write!(f, "{}", s),
      Value::List(list) => {
          let items: Vec<String> = list.borrow().iter()
              .map(|v| format!("{:?}", v))
              .collect();
          write!(f, "[{}]", items.join(", "))
      }
      Value::Function(func) => write!(f, "<function {}>", func.code.name),
      Value::NativeFunction(nf) => write!(f, "{:?}", nf),
      _ => write!(f, "<{}>", self.type_name()),
    }
  }
}