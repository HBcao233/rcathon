use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use cathon_compiler::{OpCode, CodeObject, Constant};
use crate::frame::Frame;
use crate::value::{Value, Function, NativeFn};
use crate::builtins;

pub struct VM {
  /// 调用栈
  frames: Vec<Frame>,
  /// 全局变量
  globals: Rc<RefCell<HashMap<String, Value>>>,
}

impl VM {
  pub fn new() -> Self {
    let globals = Rc::new(RefCell::new(HashMap::new()));
    
    // 注册内置函数
    let mut g = globals.borrow_mut();
    g.insert("print".to_string(), builtins::make_print());
    g.insert("len".to_string(), builtins::make_len());
    g.insert("type".to_string(), builtins::make_type());
    g.insert("range".to_string(), builtins::make_range());
    drop(g);
    
    Self {
      frames: Vec::new(),
      globals,
    }
  }

  /// 执行代码对象
  pub fn run(&mut self, code: CodeObject) -> Result<Value, RuntimeError> {
    let frame = Frame::new(code, Rc::clone(&self.globals));
    self.frames.push(frame);
    self.execute()
  }

  /// 当前帧
  fn frame(&mut self) -> &mut Frame {
    self.frames.last_mut().expect("No frame")
  }

  /// 主执行循环
  fn execute(&mut self) -> Result<Value, RuntimeError> {
    loop {
      // 读取操作码
      let opcode = OpCode::from(self.frame().read_byte());
      
      match opcode {
        // ============ 常量加载 ============
        OpCode::LoadConst => {
          let idx = self.frame().read_u16() as usize;
          let constant = self.frame().code.constants[idx].clone();
          let value = self.constant_to_value(constant);
          self.frame().push(value);
        }

        // ============ 变量操作 ============
        OpCode::LoadName => {
          let idx = self.frame().read_u16() as usize;
          let name = self.frame().code.names[idx].clone();
          
          // 先查局部，再查全局
          let value = self.globals.borrow()
            .get(&name)
            .cloned()
            .ok_or_else(|| RuntimeError::NameError(name))?;
          self.frame().push(value);
        }

        OpCode::StoreName => {
          let idx = self.frame().read_u16() as usize;
          let name = self.frame().code.names[idx].clone();
          let value = self.frame().pop();
          self.globals.borrow_mut().insert(name, value);
        }

        OpCode::LoadFast => {
          let idx = self.frame().read_u16() as usize;
          let value = self.frame().locals[idx].clone();
          self.frame().push(value);
        }

        OpCode::StoreFast => {
          let idx = self.frame().read_u16() as usize;
          let value = self.frame().pop();
          self.frame().locals[idx] = value;
        }

        // ============ 栈操作 ============
        OpCode::Pop => {
          self.frame().pop();
        }

        OpCode::Dup => {
          let value = self.frame().peek().clone();
          self.frame().push(value);
        }

        // ============ 二元运算 ============
        OpCode::BinaryAdd => {
          let right = self.frame().pop();
          let left = self.frame().pop();
          let result = self.binary_add(left, right)?;
          self.frame().push(result);
        }

        OpCode::BinarySub => {
          let right = self.frame().pop();
          let left = self.frame().pop();
          let result = self.binary_sub(left, right)?;
          self.frame().push(result);
        }

        OpCode::BinaryMul => {
          let right = self.frame().pop();
          let left = self.frame().pop();
          let result = self.binary_mul(left, right)?;
          self.frame().push(result);
        }

        OpCode::BinaryDiv => {
          let right = self.frame().pop();
          let left = self.frame().pop();
          let result = self.binary_div(left, right)?;
          self.frame().push(result);
        }

        // ============ 比较运算 ============
        OpCode::CompareEq => {
          let right = self.frame().pop();
          let left = self.frame().pop();
          let result = Value::Bool(self.equals(&left, &right));
          self.frame().push(result);
        }

        OpCode::CompareLt => {
          let right = self.frame().pop();
          let left = self.frame().pop();
          let result = self.compare_lt(left, right)?;
          self.frame().push(result);
        }

        // ============ 一元运算 ============
        OpCode::UnaryNeg => {
          let value = self.frame().pop();
          let result = match value {
              Value::Int(n) => Value::Int(-n),
              Value::Float(f) => Value::Float(-f),
              _ => return Err(RuntimeError::TypeError(
                  format!("bad operand type for unary -: '{}'", value.type_name())
              )),
          };
          self.frame().push(result);
        }

        OpCode::UnaryNot => {
          let value = self.frame().pop();
          let result = Value::Bool(!value.is_truthy());
          self.frame().push(result);
        }

        // ============ 跳转指令 ============
        OpCode::Jump => {
          let offset = self.frame().read_u16() as usize;
          self.frame().ip = offset;
        }

        OpCode::JumpIfFalse => {
          let offset = self.frame().read_u16() as usize;
          if !self.frame().peek().is_truthy() {
            self.frame().ip = offset;
          }
        }

        OpCode::JumpIfTrue => {
          let offset = self.frame().read_u16() as usize;
          if self.frame().peek().is_truthy() {
            self.frame().ip = offset;
          }
        }

        OpCode::Loop => {
          let offset = self.frame().read_u16() as usize;
          self.frame().ip = offset;
        }

        // ============ 函数相关 ============
        OpCode::MakeFunction => {
          let code = match self.frame().pop() {
            Value::None => unreachable!(),
            // 这里需要特殊处理，从常量中获取 CodeObject
            _ => unreachable!(),
          };
          // 简化处理，实际需要从栈上获取 CodeObject
        }

        OpCode::Call => {
          let argc = self.frame().read_u16() as usize;
          self.call_function(argc)?;
        }

        OpCode::Return => {
          let result = self.frame().pop();
          self.frames.pop();
          
          if self.frames.is_empty() {
              return Ok(result);
          }
          
          self.frame().push(result);
        }

        // ============ 容器操作 ============
        OpCode::BuildList => {
          let count = self.frame().read_u16() as usize;
          let mut items = Vec::with_capacity(count);
          
          for _ in 0..count {
              items.push(self.frame().pop());
          }
          items.reverse();
          
          let list = Value::List(Rc::new(RefCell::new(items)));
          self.frame().push(list);
        }

        OpCode::BinarySubscr => {
          let index = self.frame().pop();
          let obj = self.frame().pop();
          let result = self.subscript(obj, index)?;
          self.frame().push(result);
        }

        _ => {
          return Err(RuntimeError::UnknownOpcode(opcode as u8));
        }
      }
    }
  }

  fn call_function(&mut self, argc: usize) -> Result<(), RuntimeError> {
    // 收集参数
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
      args.push(self.frame().pop());
    }
    args.reverse();
    
    let callee = self.frame().pop();
    
    match callee {
      Value::Function(func) => {
        let mut frame = Frame::new(func.code.clone(), Rc::clone(&self.globals));
        
        // 绑定参数到局部变量
        for (i, arg) in args.into_iter().enumerate() {
          frame.locals[i] = arg;
        }
        
        self.frames.push(frame);
      }
      
      Value::NativeFunction(native) => {
        let result = (native.func)(args)
          .map_err(RuntimeError::NativeError)?;
        self.frame().push(result);
      }
      
      _ => {
        return Err(RuntimeError::TypeError(
          format!("'{}' object is not callable", callee.type_name())
        ));
      }
    }
    
    Ok(())
  }

  fn constant_to_value(&self, constant: Constant) -> Value {
    match constant {
      Constant::None => Value::None,
      Constant::Bool(b) => Value::Bool(b),
      Constant::Int(n) => Value::Int(n),
      Constant::Float(f) => Value::Float(f),
      Constant::String(s) => Value::String(Rc::new(s)),
      Constant::Code(code) => {
        Value::Function(Rc::new(Function {
          code: *code,
          globals: Rc::clone(&self.globals),
        }))
      }
    }
  }

  // 辅助方法
  fn binary_add(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
      (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
      (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
      (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
      (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
      (Value::String(a), Value::String(b)) => {
        Ok(Value::String(Rc::new(format!("{}{}", a, b))))
      },
      (l, r) => Err(RuntimeError::TypeError(
        format!(
          "unsupported operand type(s) for +: '{}' and '{}'", 
          l.type_name(), 
          r.type_name(),
        )
      )),
    }
  }

  fn binary_sub(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
      (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
      (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
      (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
      (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
      (l, r) => Err(RuntimeError::TypeError(
        format!(
          "unsupported operand type(s) for -: '{}' and '{}'", 
          l.type_name(), 
          r.type_name(),
        )
      )),
    }
  }

  fn binary_mul(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
      (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
      (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
      (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
      (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
      
      // "abc" * 3 = "abcabcabc"
      (Value::String(s), Value::Int(n)) | (Value::Int(n), Value::String(s)) => {
        Ok(Value::String(Rc::new(s.repeat(n.max(0) as usize))))
      },
      
      (l, r) => Err(RuntimeError::TypeError(
        format!(
          "unsupported operand type(s) for *: '{}' and '{}'", 
          l.type_name(), 
          r.type_name(),
        )
      )),
    }
  }

  fn binary_div(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (&left, &right) {
      (Value::Int(a), Value::Int(b)) => {
        if *b == 0 { return Err(RuntimeError::ZeroDivision); }
        Ok(Value::Float(*a as f64 / *b as f64))
      },
      
      (Value::Float(a), Value::Float(b)) => {
        if *b == 0.0 { return Err(RuntimeError::ZeroDivision); }
        Ok(Value::Float(a / b))
      },
      
      _ => Err(RuntimeError::TypeError(
        format!(
          "unsupported operand type(s) for /: '{}' and '{}'", 
          left.type_name(), 
          right.type_name(),
        )
      )),
    }
  }

  fn compare_lt(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match (left, right) {
      (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
      (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
      (l, r) => Err(RuntimeError::TypeError(
        format!(
          "'<' not supported between '{}' and '{}'",
          l.type_name(), 
          r.type_name(),
        )
      )),
    }
  }

  fn equals(&self, left: &Value, right: &Value) -> bool {
    match (left, right) {
      (Value::None, Value::None) => true,
      (Value::Bool(a), Value::Bool(b)) => a == b,
      (Value::Int(a), Value::Int(b)) => a == b,
      (Value::Float(a), Value::Float(b)) => a == b,
      (Value::String(a), Value::String(b)) => a == b,
      _ => false,
    }
  }

  fn subscript(&self, obj: Value, index: Value) -> Result<Value, RuntimeError> {
    match (obj, index) {
      (Value::List(list), Value::Int(i)) => {
        let list = list.borrow();
        let idx = if i < 0 {
          (list.len() as i64 + i) as usize
        } else {
          i as usize
        };
        list.get(idx).cloned()
          .ok_or(RuntimeError::IndexError)
      },
      
      (Value::String(s), Value::Int(i)) => {
        let idx = if i < 0 {
          (s.len() as i64 + i) as usize
        } else {
          i as usize
        };
        s.chars().nth(idx)
          .map(|c| Value::String(Rc::new(c.to_string())))
          .ok_or(RuntimeError::IndexError)
      },
      
      (obj, _) => Err(RuntimeError::TypeError(
        format!("'{}' object is not subscriptable", obj.type_name())
      )),
    }
  }
}

#[derive(Debug)]
pub enum RuntimeError {
  TypeError(String),
  NameError(String),
  IndexError,
  ZeroDivision,
  NativeError(String),
  UnknownOpcode(u8),
}