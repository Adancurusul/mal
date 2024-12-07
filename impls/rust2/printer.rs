use crate::MalType;

#[macro_export]
macro_rules! pr_str {
    ($exp:expr, $print_readably:expr) => {
        printer::pr_str($exp, $print_readably)
    };
}

// Convert MalType to string representation
pub fn pr_str(exp: &MalType, print_readably: bool) -> String {
    match exp {
        MalType::Nil => "nil".to_string(),
        MalType::Bool(b) => b.to_string(),
        MalType::Number(n) => n.to_string(),
        MalType::Symbol(s) => s.clone(),
        MalType::String(s) => {
            if print_readably {
                format!("\"{}\"", s.replace('\\', "\\\\")
                         .replace('\n', "\\n")
                         .replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        MalType::Keyword(k) => format!(":{}", k),
        MalType::List(items) => {
            let items: Vec<String> = items.iter()
                .map(|i| pr_str(i, print_readably))
                .collect();
            format!("({})", items.join(" "))
        }
        MalType::Vector(items) => {
            let items: Vec<String> = items.iter()
                .map(|i| pr_str(i, print_readably))
                .collect();
            format!("[{}]", items.join(" "))
        }
        MalType::Map(pairs) => {
            let items: Vec<String> = pairs.iter()
                .map(|(k, v)| format!("{} {}", 
                    pr_str(k, print_readably),
                    pr_str(v, print_readably)))
                .collect();
            format!("{{{}}}", items.join(" "))
        }
        MalType::Function { .. } => "#<function>".to_string(),
        MalType::TcoForm(..) => "#<tco>".to_string(),
    }
} 