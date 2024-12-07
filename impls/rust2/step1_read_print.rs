use std::io::{self, Write};
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
    ($input:expr) => {{
        match read($input) {
            Ok(ast) => {
                let exp = eval(&ast);
                Ok(print(&exp))
            }
            Err(e) => Err(e)
        }
    }};
}

// READ: Parse the input string into an internal data structure
fn read(input: &str) -> Result<MalType, String> {
    reader::read_str(input)
}

// EVAL: Return input ast unchanged
fn eval(ast: &MalType) -> MalType {
    ast.clone()
}

// PRINT: Convert the evaluated result back to a string
fn print(exp: &MalType) -> String {
    printer::pr_str(exp, true)
}

fn main() {
    // Print welcome message
    println!("Mal (Make-A-Lisp) Step 1: Read and Print");
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
        match rep!(&input) {
            Ok(result) => println!("{}", result),
            Err(err) => {
                if err == "Empty input" {
                    continue;
                }
                if err == "Unterminated string" {
                    eprintln!("Error: end of input");
                } else {
                    eprintln!("Error: {}", err);
                }
            }
        }
    }
} 