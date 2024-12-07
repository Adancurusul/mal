use std::io::{self, Write};
use std::collections::HashMap;
use mal_rust2::{MalType, mal};

// Import reader and printer functions
mod reader;
mod printer;

// Macro for printing the prompt and flushing stdout
#[macro_export]
macro_rules! with_prompt {
    ($prompt:expr) => {{
        print!($prompt);
        io::stdout().flush().unwrap();
    }};
}

// Macro for reading a line of input
#[macro_export]
macro_rules! read_input {
    () => {{
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }};
}

// Macro for the READ-EVAL-PRINT cycle
#[macro_export]
macro_rules! rep {
    ($input:expr, $env:expr) => {{
        match read($input) {
            Ok(ast) => {
                match eval(&ast, $env) {
                    Ok(exp) => Ok(print(&exp)),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e)
        }
    }};
}

// Macro for error handling
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($msg.to_string());
        }
    };
}

// Macro for function application
#[macro_export]
macro_rules! apply_builtin {
    ($name:expr, $args:expr, $op:tt) => {{
        ensure!($args.len() == 2, concat!($name, " requires exactly 2 arguments"));
        match (&$args[0], &$args[1]) {
            (MalType::Number(a), MalType::Number(b)) => Ok(mal!(a $op b)),
            _ => Err(concat!($name, " requires number arguments").to_string()),
        }
    }};
    ($name:expr, $args:expr, /) => {{
        ensure!($args.len() == 2, concat!($name, " requires exactly 2 arguments"));
        match (&$args[0], &$args[1]) {
            (MalType::Number(a), MalType::Number(b)) => {
                ensure!(*b != 0, "division by zero");
                Ok(mal!(a / b))
            },
            _ => Err(concat!($name, " requires number arguments").to_string()),
        }
    }};
}

// Macro for environment creation
#[macro_export]
macro_rules! env {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut env = HashMap::new();
        $(
            env.insert($k.to_string(), $v);
        )*
        env
    }};
}

// READ: Parse the input string into an internal data structure
fn read(input: &str) -> Result<MalType, String> {
    reader::read_str(input)
}

// Evaluate an AST in the given environment
fn eval_ast(ast: &MalType, env: &HashMap<String, MalType>) -> Result<MalType, String> {
    match ast {
        // For symbols, look up their value in the environment
        MalType::Symbol(sym) => {
            env.get(sym)
               .cloned()
               .ok_or_else(|| format!("Symbol '{}' not found", sym))
        }
        // For lists, evaluate each element
        MalType::List(items) => {
            let mut new_items = Vec::with_capacity(items.len());
            for item in items {
                new_items.push(eval(item, env)?);
            }
            Ok(MalType::List(new_items))
        }
        // For vectors, evaluate each element
        MalType::Vector(items) => {
            let mut new_items = Vec::with_capacity(items.len());
            for item in items {
                new_items.push(eval(item, env)?);
            }
            Ok(MalType::Vector(new_items))
        }
        // For maps, evaluate each value
        MalType::Map(pairs) => {
            let mut new_pairs = Vec::with_capacity(pairs.len());
            for (key, val) in pairs {
                new_pairs.push((key.clone(), eval(val, env)?));
            }
            Ok(MalType::Map(new_pairs))
        }
        // All other types evaluate to themselves
        _ => Ok(ast.clone()),
    }
}

// Apply a function to arguments
fn apply_function(f: &str, args: &[MalType]) -> Result<MalType, String> {
    match f {
        "+" => apply_builtin!("+", args, +),
        "-" => apply_builtin!("-", args, -),
        "*" => apply_builtin!("*", args, *),
        "/" => apply_builtin!("/", args, /),
        _ => Err(format!("Unknown function: {}", f)),
    }
}

// EVAL: Evaluate the AST
fn eval(ast: &MalType, env: &HashMap<String, MalType>) -> Result<MalType, String> {
    match ast {
        MalType::List(items) if !items.is_empty() => {
            // Evaluate the list
            let evaluated = eval_ast(ast, env)?;
            if let MalType::List(items) = evaluated {
                // Get the function and arguments
                let f = &items[0];
                let args = &items[1..];
                
                // Apply the function
                match f {
                    MalType::Symbol(s) => apply_function(s, args),
                    _ => Err("first element must be a function".to_string()),
                }
            } else {
                Ok(evaluated)
            }
        }
        _ => eval_ast(ast, env),
    }
}

// PRINT: Convert the evaluated result back to a string
fn print(exp: &MalType) -> String {
    printer::pr_str(exp, true)
}

// Create default environment with basic arithmetic functions
fn create_default_env() -> HashMap<String, MalType> {
    env! {
        // Special values
        "nil" => mal!(nil),
        "true" => mal!(true),
        "false" => mal!(false),
        // Arithmetic functions
        "+" => mal!(sym: "+"),
        "-" => mal!(sym: "-"),
        "*" => mal!(sym: "*"),
        "/" => mal!(sym: "/"),
    }
}

fn main() {
    // Create environment with basic functions
    let env = create_default_env();

    // Print welcome message
    println!("Mal (Make-A-Lisp) Step 2: Eval");
    println!("Press Ctrl+C to exit\n");

    // Main REPL loop
    loop {
        with_prompt!("user> ");
        
        let input = read_input!();
        if input.is_empty() {
            continue;
        }
        
        // Handle exit commands
        if input == "exit" || input == "quit" {
            break;
        }
        
        // Process the input and print the result
        match rep!(&input, &env) {
            Ok(result) => println!("{}", result),
            Err(err) => {
                if err == "Empty input" {
                    continue;
                }
                eprintln!("Error: {}", err);
            }
        }
    }
} 