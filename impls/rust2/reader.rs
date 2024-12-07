use crate::{MalType, mal};

// Token types for lexical analysis
#[derive(Debug, Clone)]
enum Token {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Quote,
    Quasiquote,
    Unquote,
    SpliceUnquote,
    Deref,
    Meta,
    Number(i64),
    Symbol(String),
    String(String),
    Keyword(String),
}

// Reader structure to keep track of tokens and current position
struct Reader {
    tokens: Vec<Token>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<Token>) -> Self {
        Reader {
            tokens,
            position: 0,
        }
    }

    fn next(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }
}

// Read a string, handling escape sequences
fn read_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, String> {
    let mut result = String::new();
    
    while let Some(&c) = chars.peek() {
        match c {
            '"' => {
                chars.next(); // consume closing quote
                return Ok(result);
            }
            '\\' => {
                chars.next(); // consume backslash
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(c) => result.push(c),
                    None => return Err("end of input".to_string()),
                }
            }
            _ => {
                result.push(chars.next().unwrap());
            }
        }
    }
    Err("end of input".to_string())
}

// Skip to the end of line
fn skip_comment(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c == '\n' {
            break;
        }
        chars.next();
    }
}

// Tokenize input string into a vector of tokens
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Skip whitespace and commas (commas are treated as whitespace)
            c if c.is_whitespace() || c == ',' => {
                chars.next();
            }
            // Handle comments
            ';' => {
                skip_comment(&mut chars);
            }
            // Handle strings
            '"' => {
                chars.next(); // consume opening quote
                match read_string(&mut chars) {
                    Ok(s) => tokens.push(Token::String(s)),
                    Err(e) => return Err(e),
                }
            }
            // Handle special characters
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            '\'' => {
                tokens.push(Token::Quote);
                chars.next();
            }
            '`' => {
                tokens.push(Token::Quasiquote);
                chars.next();
            }
            '~' => {
                chars.next();
                if chars.peek() == Some(&'@') {
                    chars.next();
                    tokens.push(Token::SpliceUnquote);
                } else {
                    tokens.push(Token::Unquote);
                }
            }
            '@' => {
                tokens.push(Token::Deref);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Meta);
                chars.next();
            }
            // Handle keywords
            ':' => {
                chars.next(); // consume colon
                let mut keyword = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || "+-*/<>=!?_-".contains(c) {
                        keyword.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Keyword(keyword));
            }
            // Handle numbers
            c if c.is_digit(10) || (c == '-' && chars.clone().nth(1).map_or(false, |next| next.is_digit(10))) => {
                let mut number = String::new();
                if c == '-' {
                    number.push(chars.next().unwrap());
                }
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) {
                        number.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Ok(n) = number.parse() {
                    tokens.push(Token::Number(n));
                }
            }
            // Handle symbols
            c if c.is_alphabetic() || "+-*/<>=!?_".contains(c) => {
                let mut symbol = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || "+-*/<>=!?_-:".contains(c) {
                        symbol.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Symbol(symbol));
            }
            // Skip unknown characters
            _ => {
                chars.next();
            }
        }
    }
    Ok(tokens)
}

// Read an atom (number, symbol, string, or keyword)
fn read_atom(token: Token) -> Result<MalType, String> {
    match token {
        Token::Number(n) => Ok(mal!(n)),
        Token::Symbol(s) => Ok(mal!(sym: s)),
        Token::String(s) => Ok(mal!(str: s)),
        Token::Keyword(k) => Ok(mal!(kw: k)),
        _ => Err("Invalid atom".to_string()),
    }
}

// Read a list
fn read_list(reader: &mut Reader) -> Result<MalType, String> {
    let mut items = Vec::new();
    
    loop {
        match reader.peek() {
            Some(&Token::RightParen) => {
                reader.next();
                return Ok(MalType::List(items));
            }
            Some(_) => {
                items.push(read_form(reader)?);
            }
            None => {
                return Err("end of input".to_string());
            }
        }
    }
}

// Read a vector
fn read_vector(reader: &mut Reader) -> Result<MalType, String> {
    let mut items = Vec::new();
    
    loop {
        match reader.peek() {
            Some(&Token::RightBracket) => {
                reader.next();
                return Ok(MalType::Vector(items));
            }
            Some(_) => {
                items.push(read_form(reader)?);
            }
            None => {
                return Err("end of input".to_string());
            }
        }
    }
}

// Read a hash map
fn read_hash_map(reader: &mut Reader) -> Result<MalType, String> {
    let mut pairs = Vec::new();
    
    loop {
        match reader.peek() {
            Some(&Token::RightBrace) => {
                reader.next();
                return Ok(MalType::Map(pairs));
            }
            Some(_) => {
                let key = read_form(reader)?;
                match reader.peek() {
                    Some(_) => {
                        let value = read_form(reader)?;
                        pairs.push((key, value));
                    }
                    None => {
                        return Err("end of input".to_string());
                    }
                }
            }
            None => {
                return Err("end of input".to_string());
            }
        }
    }
}

// Read any form
fn read_form(reader: &mut Reader) -> Result<MalType, String> {
    match reader.next() {
        Some(Token::Quote) => {
            let form = read_form(reader)?;
            Ok(mal!(list: mal!(sym: "quote"), form))
        }
        Some(Token::Quasiquote) => {
            let form = read_form(reader)?;
            Ok(mal!(list: mal!(sym: "quasiquote"), form))
        }
        Some(Token::Unquote) => {
            let form = read_form(reader)?;
            Ok(mal!(list: mal!(sym: "unquote"), form))
        }
        Some(Token::SpliceUnquote) => {
            let form = read_form(reader)?;
            Ok(mal!(list: mal!(sym: "splice-unquote"), form))
        }
        Some(Token::Deref) => {
            let form = read_form(reader)?;
            Ok(mal!(list: mal!(sym: "deref"), form))
        }
        Some(Token::Meta) => {
            let meta = read_form(reader)?;
            let form = read_form(reader)?;
            Ok(mal!(list: mal!(sym: "with-meta"), form, meta))
        }
        Some(Token::LeftParen) => read_list(reader),
        Some(Token::LeftBracket) => read_vector(reader),
        Some(Token::LeftBrace) => read_hash_map(reader),
        Some(token) => read_atom(token),
        None => Err("end of input".to_string()),
    }
}

// Main entry point for the reader
pub fn read_str(input: &str) -> Result<MalType, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty input".to_string());
    }
    
    let mut reader = Reader::new(tokens);
    read_form(&mut reader)
} 