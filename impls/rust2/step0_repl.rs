use std::io::{self, Write};
use mal_rust2::{MalType, mal};

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

// READ: Return input string unchanged
fn read(input: &str) -> MalType {
    mal!(str: input.to_string())
}

// EVAL: Return input string unchanged
fn eval(ast: &MalType) -> MalType {
    ast.clone()
}

// PRINT: Return input string unchanged
fn print(exp: &MalType) -> String {
    exp.print()
}

// Macro for the READ-EVAL-PRINT cycle
#[macro_export]
macro_rules! rep {
    ($input:expr) => {{
        let ast = read($input);
        let exp = eval(&ast);
        print(&exp)
    }};
}

fn main() {
    // Print welcome message
    println!("Mal (Make-A-Lisp) Step 0: REPL");
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
        println!("{}", rep!(&input));
    }
}
