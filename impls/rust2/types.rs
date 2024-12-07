#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl std::fmt::Display for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MalType::Nil => write!(f, "nil"),
            MalType::Bool(b) => write!(f, "{}", b),
            MalType::Number(n) => write!(f, "{}", n),
            MalType::Symbol(s) => write!(f, "{}", s),
            MalType::String(s) => write!(f, "\"{}\"", s),
            MalType::Keyword(k) => write!(f, ":{}", k),
            MalType::List(items) => {
                write!(f, "(")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, ")")
            }
            MalType::Vector(items) => {
                write!(f, "[")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, "]")
            }
            MalType::Map(pairs) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in pairs {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            }
        }
    }
}

// Basic type macros
#[macro_export]
macro_rules! mal {
    (nil) => { $crate::MalType::Nil };
    (true) => { $crate::MalType::Bool(true) };
    (false) => { $crate::MalType::Bool(false) };
    ($n:expr) => { $crate::MalType::Number($n) };
    (sym: $s:expr) => { $crate::MalType::Symbol($s.to_string()) };
    (str: $s:expr) => { $crate::MalType::String($s.to_string()) };
    (kw: $s:expr) => { $crate::MalType::Keyword($s.to_string()) };
    (list: $($x:expr),*) => { $crate::MalType::List(vec![$($x),*]) };
    (vec: $($x:expr),*) => { $crate::MalType::Vector(vec![$($x),*]) };
    (map: $($k:expr => $v:expr),*) => { 
        $crate::MalType::Map(vec![$(($k, $v)),*])
    };
}

// List construction macro
#[macro_export]
macro_rules! list {
    ($($x:expr),*) => { mal!(list: $($x),*) };
}

// Vector construction macro
#[macro_export]
macro_rules! vector {
    ($($x:expr),*) => { mal!(vec: $($x),*) };
}

// Map construction macro
#[macro_export]
macro_rules! hashmap {
    ($($k:expr => $v:expr),*) => { mal!(map: $($k => $v),*) };
}

// Symbol construction macro
#[macro_export]
macro_rules! symbol {
    ($s:expr) => { mal!(sym: $s) };
}

// Keyword construction macro
#[macro_export]
macro_rules! keyword {
    ($s:expr) => { mal!(kw: $s) };
}

// String construction macro
#[macro_export]
macro_rules! string {
    ($s:expr) => { mal!(str: $s) };
}

// Special form construction macro
#[macro_export]
macro_rules! special_form {
    (quote, $ast:expr) => { list!(symbol!("quote"), $ast) };
    (quasiquote, $ast:expr) => { list!(symbol!("quasiquote"), $ast) };
    (unquote, $ast:expr) => { list!(symbol!("unquote"), $ast) };
    (splice_unquote, $ast:expr) => { list!(symbol!("splice-unquote"), $ast) };
    (deref, $ast:expr) => { list!(symbol!("deref"), $ast) };
    (with_meta, $obj:expr, $meta:expr) => { list!(symbol!("with-meta"), $obj, $meta) };
}

impl MalType {
    // Convert MalType to its string representation
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
                let items: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{} {}", k.print(), v.print()))
                    .collect();
                format!("{{{}}}", items.join(" "))
            }
        }
    }
}
