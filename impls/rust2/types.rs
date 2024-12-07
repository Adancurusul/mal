#[derive(Debug, Clone, PartialEq)]
pub enum MalType {
    Nil,
    Bool(bool),
    Number(i64),
    Symbol(String),
    String(String),
    List(Vec<MalType>),
    Vector(Vec<MalType>),
    Map(Vec<(MalType, MalType)>),
}

// Macro for creating MalType values with less boilerplate
#[macro_export]
macro_rules! mal {
    // Nil value
    (nil) => { $crate::MalType::Nil };
    // Boolean values
    (true) => { $crate::MalType::Bool(true) };
    (false) => { $crate::MalType::Bool(false) };
    // Numbers
    ($n:expr) => { $crate::MalType::Number($n) };
    // Strings and symbols with type specifier
    (str: $s:expr) => { $crate::MalType::String($s.to_string()) };
    (sym: $s:expr) => { $crate::MalType::Symbol($s.to_string()) };
    // Lists with variable number of elements
    (list: $($x:expr),* $(,)?) => { 
        $crate::MalType::List(vec![$($x),*]) 
    };
    // Vectors with variable number of elements
    (vec: $($x:expr),* $(,)?) => { 
        $crate::MalType::Vector(vec![$($x),*]) 
    };
    // Maps with key-value pairs
    (map: $($k:expr => $v:expr),* $(,)?) => {
        $crate::MalType::Map(vec![$(($k, $v)),*])
    };
}

// For backward compatibility
#[macro_export]
macro_rules! mal_string {
    ($s:expr) => { mal!(str: $s) };
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
