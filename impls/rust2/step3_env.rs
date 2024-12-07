use std::io::{self, Write};
use std::rc::Rc;
use std::cell::RefCell;
use mal_rust2::{MalType, mal};

// Import modules
mod reader;
mod printer;
mod env;
use env::Env;

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
            (MalType::Number(a), MalType::Number(b)) => Ok(mal!(op: *a, $op, *b)),
            _ => Err(concat!($name, " requires number arguments").to_string()),
        }
    }};
}

// Macro for special form handling
#[macro_export]
macro_rules! handle_special {
    ($ast:expr, $env:expr, def!) => {{
        if $ast.len() != 3 {
            Err("def! requires exactly 2 arguments".to_string())
        } else if let MalType::Symbol(key) = &$ast[1] {
            match eval(&$ast[2], $env) {
                Ok(value) => {
                    $env.borrow_mut().set(key, value.clone());
                    Ok(value)
                }
                Err(e) => Err(e),
            }
        } else {
            Err("def! first argument must be a symbol".to_string())
        }
    }};
    ($ast:expr, $env:expr, let*) => {{
        if $ast.len() != 3 {
            Err("let* requires exactly 2 arguments".to_string())
        } else {
            let new_env = env_new!(Some($env.clone()));
            
            match &$ast[1] {
                MalType::List(bindings) | MalType::Vector(bindings) => {
                    if bindings.len() % 2 != 0 {
                        Err("let* requires an even number of binding forms".to_string())
                    } else {
                        for chunk in bindings.chunks(2) {
                            if let MalType::Symbol(key) = &chunk[0] {
                                match eval(&chunk[1], &new_env) {
                                    Ok(value) => {
                                        new_env.borrow_mut().set(key, value);
                                    }
                                    Err(e) => return Err(e),
                                }
                            } else {
                                return Err("let* binding key must be a symbol".to_string());
                            }
                        }
                        
                        eval(&$ast[2], &new_env)
                    }
                }
                _ => Err("let* first argument must be a list or vector".to_string()),
            }
        }
    }};
}

// Macro for creating a new environment
#[macro_export]
macro_rules! env_new {
    ($outer:expr) => {
        Rc::new(RefCell::new(Env::new($outer)))
    };
    () => {
        env_new!(None)
    };
}

// Macro for setting multiple bindings in an environment
#[macro_export]
macro_rules! env_bind {
    ($env:expr, $($key:expr => $val:expr),* $(,)?) => {{
        $(
            $env.borrow_mut().set($key, $val);
        )*
    }};
}

// READ: Parse the input string into an internal data structure
fn read(input: &str) -> Result<MalType, String> {
    reader::read_str(input)
}

// Evaluate an AST in the given environment
fn eval_ast(ast: &MalType, env: &Rc<RefCell<Env>>) -> Result<MalType, String> {
    match ast {
        MalType::Symbol(sym) => {
            env.borrow()
               .get(sym)
               .ok_or_else(|| format!("Symbol '{}' not found", sym))
        }
        MalType::List(items) => {
            let mut new_items = Vec::with_capacity(items.len());
            for item in items {
                new_items.push(eval(item, env)?);
            }
            Ok(MalType::List(new_items))
        }
        MalType::Vector(items) => {
            let mut new_items = Vec::with_capacity(items.len());
            for item in items {
                new_items.push(eval(item, env)?);
            }
            Ok(MalType::Vector(new_items))
        }
        MalType::Map(pairs) => {
            let mut new_pairs = Vec::with_capacity(pairs.len());
            for (key, val) in pairs {
                new_pairs.push((key.clone(), eval(val, env)?));
            }
            Ok(MalType::Map(new_pairs))
        }
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

// Handle special forms (def! and let*)
fn handle_special_form(ast: &[MalType], env: &Rc<RefCell<Env>>) -> Result<MalType, String> {
    if let Some(MalType::Symbol(sym)) = ast.first() {
        match sym.as_str() {
            "def!" => handle_special!(ast, env, def!),
            "let*" => handle_special!(ast, env, let*),
            _ => {
                let evaluated = eval_ast(&MalType::List(ast.to_vec()), env)?;
                if let MalType::List(items) = evaluated {
                    if items.is_empty() {
                        return Ok(MalType::List(vec![]));
                    }
                    let f = &items[0];
                    let args = &items[1..];
                    match f {
                        MalType::Symbol(s) => apply_function(s, args),
                        _ => Err("first element must be a function".to_string()),
                    }
                } else {
                    Ok(evaluated)
                }
            }
        }
    } else {
        eval_ast(&MalType::List(ast.to_vec()), env)
    }
}

// EVAL: Evaluate the AST
fn eval(ast: &MalType, env: &Rc<RefCell<Env>>) -> Result<MalType, String> {
    // Check if DEBUG-EVAL is enabled
    let debug = match env.borrow().get("DEBUG-EVAL") {
        Some(MalType::Bool(true)) | Some(MalType::Number(_)) | Some(MalType::String(_)) | Some(MalType::List(_)) => true,
        _ => false,
    };

    if debug {
        eprintln!("EVAL: {}", printer::pr_str(ast, true));
    }

    let result = match ast {
        MalType::List(items) if !items.is_empty() => {
            // Check for special forms first
            handle_special_form(items, env)
        }
        _ => eval_ast(ast, env),
    };

    if debug {
        if let Ok(ref value) = result {
            eprintln!("{}", printer::pr_str(value, true));
        }
    }

    result
}

// PRINT: Convert the evaluated result back to a string
fn print(exp: &MalType) -> String {
    printer::pr_str(exp, true)
}

// Create default environment with basic arithmetic functions
fn create_default_env() -> Rc<RefCell<Env>> {
    let env = env_new!();
    env_bind!(env,
        // Special values
        "nil" => mal!(nil),
        "true" => mal!(true),
        "false" => mal!(false),
        // Arithmetic functions
        "+" => mal!(sym: "+"),
        "-" => mal!(sym: "-"),
        "*" => mal!(sym: "*"),
        "/" => mal!(sym: "/"),
    );
    env
}

fn main() {
    // Create environment with basic functions
    let env = create_default_env();

    // Print welcome message
    println!("Mal (Make-A-Lisp) Step 3: Environments");
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
