use crate::value::{Value, NativeFn};
use std::rc::Rc;

pub fn make_print() -> Value {
    Value::NativeFunction(NativeFn {
        name: "print".to_string(),
        func: |args| {
            let output: Vec<String> = args.iter()
                .map(|v| format!("{}", v))
                .collect();
            println!("{}", output.join(" "));
            Ok(Value::None)
        },
    })
}

pub fn make_len() -> Value {
    Value::NativeFunction(NativeFn {
        name: "len".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("len() takes exactly one argument".to_string());
            }
            match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                Value::List(list) => Ok(Value::Int(list.borrow().len() as i64)),
                other => Err(format!("object of type '{}' has no len()", other.type_name())),
            }
        },
    })
}

pub fn make_type() -> Value {
    Value::NativeFunction(NativeFn {
        name: "type".to_string(),
        func: |args| {
            if args.len() != 1 {
                return Err("type() takes exactly one argument".to_string());
            }
            Ok(Value::String(Rc::new(format!("<class '{}'>", args[0].type_name()))))
        },
    })
}

pub fn make_range() -> Value {
    Value::NativeFunction(NativeFn {
        name: "range".to_string(),
        func: |args| {
            let (start, end) = match args.len() {
                1 => match &args[0] {
                    Value::Int(n) => (0, *n),
                    _ => return Err("range() integer expected".to_string()),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::Int(a), Value::Int(b)) => (*a, *b),
                    _ => return Err("range() integers expected".to_string()),
                },
                _ => return Err("range() takes 1 or 2 arguments".to_string()),
            };
            
            let list: Vec<Value> = (start..end).map(Value::Int).collect();
            Ok(Value::List(Rc::new(std::cell::RefCell::new(list))))
        },
    })
}

pub fn make_input() -> Value {
    Value::NativeFunction(NativeFn {
        name: "input".to_string(),
        func: |args| {
            if let Some(Value::String(prompt)) = args.first() {
                print!("{}", prompt);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            Ok(Value::String(Rc::new(input.trim().to_string())))
        },
    })
}