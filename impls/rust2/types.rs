pub mod env;

#[derive(Debug, Clone, PartialEq)]
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
}

impl MalType {
    pub fn print(&self) -> String {
        match self {
            MalType::Nil => "nil".to_string(),
            MalType::Bool(b) => b.to_string(),
            MalType::Number(n) => n.to_string(),
            MalType::Symbol(s) => s.clone(),
            MalType::String(s) => s.clone(),
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
        }
    }
}

pub use crate::env::Env;
