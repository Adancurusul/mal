use std::io::{self, Write};
use std::rc::Rc;
use std::cell::RefCell;
use mal_rust2::{MalType, mal, env_new, env_bind, Env, is_type, get_value, ensure_type, apply_fn, ensure};

// Import modules
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

// Macro for special form handling
#[macro_export]
macro_rules! handle_special {
    ($ast:expr, $env:expr, fn*) => {{
        if $ast.len() != 3 {
            Err("fn* requires exactly 2 arguments".to_string())
        } else {
            match &$ast[1] {
                MalType::List(params) | MalType::Vector(params) => {
                    let mut param_names = Vec::new();
                    let mut i = 0;
                    
                    while i < params.len() {
                        match &params[i] {
                            MalType::Symbol(name) if name == "&" => {
                                if i + 1 >= params.len() {
                                    return Err("& must be followed by a symbol".to_string());
                                }
                                if let MalType::Symbol(rest_name) = &params[i + 1] {
                                    param_names.push("&".to_string());
                                    param_names.push(rest_name.clone());
                                    i += 2;
                                } else {
                                    return Err("& must be followed by a symbol".to_string());
                                }
                            }
                            MalType::Symbol(name) => {
                                param_names.push(name.clone());
                                i += 1;
                            }
                            _ => return Err("fn* parameters must be symbols".to_string()),
                        }
                    }
                    
                    Ok(mal!(fn: param_names, $ast[2].clone(), $env.clone()))
                }
                _ => Err("fn* first argument must be a list or vector".to_string()),
            }
        }
    }};
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
    ($ast:expr, $env:expr, do) => {{
        if $ast.len() < 2 {
            Err("do requires at least one argument".to_string())
        } else {
            let mut result = Ok(mal!(nil));
            for expr in $ast[1..].iter() {
                result = eval(expr, $env);
                if result.is_err() {
                    break;
                }
            }
            result
        }
    }};
    ($ast:expr, $env:expr, if) => {{
        if $ast.len() != 3 && $ast.len() != 4 {
            Err("if requires 2 or 3 arguments".to_string())
        } else {
            match eval(&$ast[1], $env)? {
                MalType::Bool(false) | MalType::Nil => {
                    if $ast.len() == 4 {
                        eval(&$ast[3], $env)
                    } else {
                        Ok(mal!(nil))
                    }
                }
                _ => eval(&$ast[2], $env),
            }
        }
    }};
}

// READ: Parse the input string into an internal data structure
fn read(input: &str) -> Result<MalType, String> {
    reader::read_str(input)
}

// Apply a function to arguments
fn apply_function(f: &MalType, args: &[MalType], env: &Rc<RefCell<Env>>) -> Result<MalType, String> {
    match f {
        MalType::Function { .. } => apply_fn!(f, args, env),
        MalType::Symbol(s) => {
            match s.as_str() {
                // Arithmetic functions
                "+" => {
                    ensure!(args.len() == 2, "+ requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(a + b))
                }
                "-" => {
                    ensure!(args.len() == 2, "- requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(a - b))
                }
                "*" => {
                    ensure!(args.len() == 2, "* requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(a * b))
                }
                "/" => {
                    ensure!(args.len() == 2, "/ requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    if b == 0 {
                        Err("division by zero".to_string())
                    } else {
                        Ok(mal!(a / b))
                    }
                }

                // Comparison functions
                "=" => {
                    ensure!(args.len() == 2, "= requires exactly 2 arguments");
                    Ok(mal!(bool: args[0] == args[1]))
                }
                ">" => {
                    ensure!(args.len() == 2, "> requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(bool: a > b))
                }
                ">=" => {
                    ensure!(args.len() == 2, ">= requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(bool: a >= b))
                }
                "<" => {
                    ensure!(args.len() == 2, "< requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(bool: a < b))
                }
                "<=" => {
                    ensure!(args.len() == 2, "<= requires exactly 2 arguments");
                    let a = get_value!(&args[0], number)?;
                    let b = get_value!(&args[1], number)?;
                    Ok(mal!(bool: a <= b))
                }

                // List functions
                "list" => Ok(MalType::List(args.to_vec())),
                "list?" => {
                    ensure!(args.len() == 1, "list? requires exactly 1 argument");
                    Ok(mal!(bool: is_type!(&args[0], list)))
                }
                "empty?" => {
                    ensure!(args.len() == 1, "empty? requires exactly 1 argument");
                    match &args[0] {
                        MalType::List(items) | MalType::Vector(items) => Ok(mal!(bool: items.is_empty())),
                        _ => Err("empty? requires a list or vector argument".to_string()),
                    }
                }
                "count" => {
                    ensure!(args.len() == 1, "count requires exactly 1 argument");
                    match &args[0] {
                        MalType::List(items) | MalType::Vector(items) => Ok(mal!(items.len() as i64)),
                        MalType::Nil => Ok(mal!(0)),
                        _ => Err("count requires a list, vector, or nil argument".to_string()),
                    }
                }

                // String functions
                "pr-str" => {
                    let strs: Vec<String> = args.iter()
                        .map(|arg| printer::pr_str(arg, true))
                        .collect();
                    Ok(mal!(str: strs.join(" ")))
                }
                "str" => {
                    let strs: Vec<String> = args.iter()
                        .map(|arg| printer::pr_str(arg, false))
                        .collect();
                    Ok(mal!(str: strs.join("")))
                }
                "prn" => {
                    let strs: Vec<String> = args.iter()
                        .map(|arg| printer::pr_str(arg, true))
                        .collect();
                    println!("{}", strs.join(" "));
                    Ok(mal!(nil))
                }
                "println" => {
                    let strs: Vec<String> = args.iter()
                        .map(|arg| printer::pr_str(arg, false))
                        .collect();
                    println!("{}", strs.join(" "));
                    Ok(mal!(nil))
                }

                // Other functions
                "not" => {
                    ensure!(args.len() == 1, "not requires exactly 1 argument");
                    match &args[0] {
                        MalType::Bool(false) | MalType::Nil => Ok(mal!(true)),
                        _ => Ok(mal!(false)),
                    }
                }

                _ => {
                    if let Some(val) = env.borrow().get(s) {
                        match val {
                            MalType::Function { .. } => apply_fn!(&val, args, env),
                            _ => Err(format!("Symbol '{}' is not a function", s)),
                        }
                    } else {
                        Err(format!("Unknown function: {}", s))
                    }
                }
            }
        }
        _ => Err("first element must be a function".to_string()),
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
            let first = &items[0];
            match first {
                MalType::Symbol(sym) => {
                    match sym.as_str() {
                        "def!" => handle_special!(&items, env, def!),
                        "let*" => handle_special!(&items, env, let*),
                        "do" => handle_special!(&items, env, do),
                        "if" => handle_special!(&items, env, if),
                        "fn*" => handle_special!(&items, env, fn*),
                        _ => {
                            let mut evaluated = Vec::new();
                            for expr in items {
                                evaluated.push(eval(expr, env)?);
                            }
                            let f = &evaluated[0];
                            let args = &evaluated[1..];
                            apply_function(f, args, env)
                        }
                    }
                }
                _ => {
                    let mut evaluated = Vec::new();
                    for expr in items {
                        evaluated.push(eval(expr, env)?);
                    }
                    let f = &evaluated[0];
                    let args = &evaluated[1..];
                    apply_function(f, args, env)
                }
            }
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
        // Comparison functions
        "=" => mal!(sym: "="),
        ">" => mal!(sym: ">"),
        ">=" => mal!(sym: ">="),
        "<" => mal!(sym: "<"),
        "<=" => mal!(sym: "<="),
        // List functions
        "list" => mal!(sym: "list"),
        "list?" => mal!(sym: "list?"),
        "empty?" => mal!(sym: "empty?"),
        "count" => mal!(sym: "count"),
        // String functions
        "pr-str" => mal!(sym: "pr-str"),
        "str" => mal!(sym: "str"),
        "prn" => mal!(sym: "prn"),
        "println" => mal!(sym: "println"),
        // Other functions
        "not" => mal!(sym: "not"),
    );
    env
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

fn main() {
    // Create environment with basic functions
    let env = create_default_env();

    // Print welcome message
    println!("Mal (Make-A-Lisp) Step 4: if, fn & do");
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