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

// Macro for the READ-EVAL-PRINT cycle
// Takes an expression and returns its evaluated result
#[macro_export]
macro_rules! rep {
    ($input:expr) => {{
        let ast = read($input);    // READ phase
        let exp = eval(&ast);      // EVAL phase
        print(&exp)                // PRINT phase
    }};
}

// READ: Parse the input string into an internal data structure
fn read(input: &str) -> MalType {
    mal!(str: input)
}

// EVAL: Evaluate the internal data structure
fn eval(ast: &MalType) -> MalType {
    ast.clone()
}

// PRINT: Convert the evaluated result back to a string
fn print(exp: &MalType) -> String {
    exp.print()
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
