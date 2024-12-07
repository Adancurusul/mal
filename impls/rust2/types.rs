pub mod env;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub enum MalType {
    Nil,
    Bool(bool),
    Number(i64),
    Symbol(String),
    String(String),
    Keyword(String),
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Map(Vec<(MalType, MalType)>),
    Function {
        params: Vec<String>,
        body: Box<MalType>,
        env: Rc<RefCell<env::Env>>,
        is_macro: bool,
    },
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MalType::Nil, MalType::Nil) => true,
            (MalType::Bool(a), MalType::Bool(b)) => a == b,
            (MalType::Number(a), MalType::Number(b)) => a == b,
            (MalType::Symbol(a), MalType::Symbol(b)) => a == b,
            (MalType::String(a), MalType::String(b)) => a == b,
            (MalType::Keyword(a), MalType::Keyword(b)) => a == b,
            (MalType::List(a), MalType::List(b)) | 
            (MalType::Vector(a), MalType::Vector(b)) |
            (MalType::List(a), MalType::Vector(b)) |
            (MalType::Vector(a), MalType::List(b)) => a == b,
            (MalType::Map(a), MalType::Map(b)) => a == b,
            (MalType::Function { .. }, MalType::Function { .. }) => false, // Functions are never equal
            _ => false,
        }
    }
}

// Macro for creating MalType values
#[macro_export]
macro_rules! mal {
    (nil) => { MalType::Nil };
    (true) => { MalType::Bool(true) };
    (false) => { MalType::Bool(false) };
    (bool: $b:expr) => { MalType::Bool($b) };
    ($n:expr) => { MalType::Number($n) };
    (str: $s:expr) => { MalType::String($s) };
    (sym: $s:expr) => { MalType::Symbol($s.to_string()) };
    (key: $s:expr) => { MalType::Keyword($s.to_string()) };
    (list: $($x:expr),* $(,)?) => { MalType::List(vec![$($x),*]) };
    (vec: $($x:expr),* $(,)?) => { MalType::Vector(vec![$($x),*]) };
    (map: $($k:expr => $v:expr),* $(,)?) => {
        MalType::Map(vec![$(($k, $v)),*])
    };
    (op: $a:expr, $op:tt, $b:expr) => {
        MalType::Number($a $op $b)
    };
    (fn: $params:expr, $body:expr, $env:expr) => {
        MalType::Function {
            params: $params,
            body: Box::new($body),
            env: $env,
            is_macro: false,
        }
    };
}

// Macro for type checking
#[macro_export]
macro_rules! is_type {
    ($val:expr, nil) => {
        matches!($val, MalType::Nil)
    };
    ($val:expr, bool) => {
        matches!($val, MalType::Bool(_))
    };
    ($val:expr, number) => {
        matches!($val, MalType::Number(_))
    };
    ($val:expr, symbol) => {
        matches!($val, MalType::Symbol(_))
    };
    ($val:expr, string) => {
        matches!($val, MalType::String(_))
    };
    ($val:expr, keyword) => {
        matches!($val, MalType::Keyword(_))
    };
    ($val:expr, list) => {
        matches!($val, MalType::List(_))
    };
    ($val:expr, vector) => {
        matches!($val, MalType::Vector(_))
    };
    ($val:expr, map) => {
        matches!($val, MalType::Map(_))
    };
    ($val:expr, function) => {
        matches!($val, MalType::Function { .. })
    };
}

// Macro for extracting values
#[macro_export]
macro_rules! get_value {
    ($val:expr, bool) => {
        match $val {
            MalType::Bool(b) => Ok(*b),
            _ => Err(format!("Expected boolean, got {:?}", $val)),
        }
    };
    ($val:expr, number) => {
        match $val {
            MalType::Number(n) => Ok(*n),
            _ => Err(format!("Expected number, got {:?}", $val)),
        }
    };
    ($val:expr, string) => {
        match $val {
            MalType::String(s) => Ok(s.clone()),
            _ => Err(format!("Expected string, got {:?}", $val)),
        }
    };
    ($val:expr, symbol) => {
        match $val {
            MalType::Symbol(s) => Ok(s.clone()),
            _ => Err(format!("Expected symbol, got {:?}", $val)),
        }
    };
    ($val:expr, list) => {
        match $val {
            MalType::List(l) => Ok(l.clone()),
            _ => Err(format!("Expected list, got {:?}", $val)),
        }
    };
    ($val:expr, vector) => {
        match $val {
            MalType::Vector(v) => Ok(v.clone()),
            _ => Err(format!("Expected vector, got {:?}", $val)),
        }
    };
}

// Macro for error handling
#[macro_export]
macro_rules! ensure_type {
    ($val:expr, $type:ident) => {
        if !is_type!($val, $type) {
            return Err(format!("Expected {}, got {:?}", stringify!($type), $val));
        }
    };
}

// Macro for function application
#[macro_export]
macro_rules! apply_fn {
    ($f:expr, $args:expr, $env:expr) => {{
        match $f {
            MalType::Function { params, body, env: fn_env, .. } => {
                let new_env = env_new!(Some(fn_env.clone()));
                let mut i = 0;
                let mut rest_args = Vec::new();
                let mut is_variadic = false;
                
                if params.len() >= 2 && params[params.len() - 2] == "&" {
                    is_variadic = true;
                }
                
                let regular_params_len = if is_variadic { params.len() - 2 } else { params.len() };
                while i < regular_params_len {
                    if i < $args.len() {
                        new_env.borrow_mut().set(&params[i], $args[i].clone());
                    } else {
                        new_env.borrow_mut().set(&params[i], mal!(nil));
                    }
                    i += 1;
                }
                
                if is_variadic {
                    rest_args.extend_from_slice(&$args[i..]);
                    new_env.borrow_mut().set(&params[params.len() - 1], MalType::List(rest_args));
                }
                
                eval(&*body, &new_env)
            }
            _ => Err(format!("Expected function, got {:?}", $f)),
        }
    }};
}

impl MalType {
    pub fn print(&self) -> String {
        match self {
            MalType::Nil => "nil".to_string(),
            MalType::Bool(b) => b.to_string(),
            MalType::Number(n) => n.to_string(),
            MalType::Symbol(s) => s.clone(),
            MalType::String(s) => format!("\"{}\"", s.replace('\\', "\\\\")
                                                    .replace('\n', "\\n")
                                                    .replace('"', "\\\"")),
            MalType::Keyword(k) => format!(":{}", k),
            MalType::List(items) => {
                let items: Vec<String> = items.iter().map(|i| i.print()).collect();
                format!("({})", items.join(" "))
            }
            MalType::Vector(items) => {
                let items: Vec<String> = items.iter().map(|i| i.print()).collect();
                format!("[{}]", items.join(" "))
            }
            MalType::Map(pairs) => {
                let items: Vec<String> = pairs.iter()
                    .map(|(k, v)| format!("{} {}", k.print(), v.print()))
                    .collect();
                format!("{{{}}}", items.join(" "))
            }
            MalType::Function { .. } => "#<function>".to_string(),
        }
    }
}

// Re-export Env for use by other modules
pub use env::Env;

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.to_string());
        }
    };
}
