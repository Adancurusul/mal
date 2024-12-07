use std::iter::Peekable;
use std::str::Chars;
use crate::MalType;
use crate::mal;

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
    WithMeta,
    String(String),
    Number(i64),
    Symbol(String),
    Keyword(String),
    Bool(bool),
    Nil,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Skip whitespace
            c if c.is_whitespace() => {
                chars.next();
            }
            // Skip comments
            ';' => {
                while let Some(c) = chars.next() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            // Special characters
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
                tokens.push(Token::WithMeta);
                chars.next();
            }
            '"' => {
                chars.next(); // Skip opening quote
                let mut string = String::new();
                let mut escaped = false;

                while let Some(c) = chars.next() {
                    if escaped {
                        match c {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),
                            _ => return Err(format!("Invalid escape sequence: \\{}", c)),
                        }
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        tokens.push(Token::String(string));
                        break;
                    } else {
                        string.push(c);
                    }
                }

                if escaped {
                    return Err("String ended while parsing escape sequence".to_string());
                }
            }
            ':' => {
                chars.next(); // Skip colon
                let mut keyword = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '-' {
                        keyword.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Keyword(keyword));
            }
            // Numbers
            c if c.is_digit(10) || (c == '-' && chars.clone().nth(1).map_or(false, |next| next.is_digit(10))) => {
                let mut number = String::new();
                if c == '-' {
                    number.push(c);
                    chars.next();
                }
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) {
                        number.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match number.parse() {
                    Ok(n) => tokens.push(Token::Number(n)),
                    Err(_) => return Err(format!("Invalid number: {}", number)),
                }
            }
            // Symbols and special values
            c if c.is_alphabetic() || "+-/*_<>=!?".contains(c) => {
                let mut symbol = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || "+-/*_<>=!?".contains(c) {
                        symbol.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match symbol.as_str() {
                    "nil" => tokens.push(Token::Nil),
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    _ => tokens.push(Token::Symbol(symbol)),
                }
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }

    Ok(tokens)
}

fn read_form(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<MalType, String> {
    match tokens.peek() {
        Some(Token::LeftParen) => read_list(tokens),
        Some(Token::LeftBracket) => read_vector(tokens),
        Some(Token::LeftBrace) => read_map(tokens),
        Some(_) => read_atom(tokens),
        None => Err("Unexpected end of input".to_string()),
    }
}

fn read_list(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<MalType, String> {
    tokens.next(); // Skip left paren
    let mut items = Vec::new();
    
    while let Some(token) = tokens.peek() {
        if let Token::RightParen = token {
            tokens.next();
            return Ok(MalType::List(items));
        }
        items.push(read_form(tokens)?);
    }
    
    Err("Expected closing parenthesis".to_string())
}

fn read_vector(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<MalType, String> {
    tokens.next(); // Skip left bracket
    let mut items = Vec::new();
    
    while let Some(token) = tokens.peek() {
        if let Token::RightBracket = token {
            tokens.next();
            return Ok(MalType::Vector(items));
        }
        items.push(read_form(tokens)?);
    }
    
    Err("Expected closing bracket".to_string())
}

fn read_map(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<MalType, String> {
    tokens.next(); // Skip left brace
    let mut pairs = Vec::new();
    
    while let Some(token) = tokens.peek() {
        if let Token::RightBrace = token {
            tokens.next();
            return Ok(MalType::Map(pairs));
        }
        let key = read_form(tokens)?;
        let value = read_form(tokens)?;
        pairs.push((key, value));
    }
    
    Err("Expected closing brace".to_string())
}

fn read_atom(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<MalType, String> {
    match tokens.next() {
        Some(Token::Nil) => Ok(mal!(nil)),
        Some(Token::Bool(b)) => Ok(mal!(bool: *b)),
        Some(Token::Number(n)) => Ok(mal!(*n)),
        Some(Token::String(s)) => Ok(mal!(str: s.clone())),
        Some(Token::Symbol(s)) => Ok(mal!(sym: s.clone())),
        Some(Token::Keyword(k)) => Ok(mal!(key: k.clone())),
        _ => Err("Invalid atom".to_string()),
    }
}

pub fn read_str(input: &str) -> Result<MalType, String> {
    if input.trim().is_empty() {
        return Err("Empty input".to_string());
    }
    
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("No valid tokens".to_string());
    }
    
    let mut token_iter = tokens.iter().peekable();
    read_form(&mut token_iter)
} 