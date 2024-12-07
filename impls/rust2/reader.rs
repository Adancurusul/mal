use std::str::Chars;
use crate::MalType;

// Token types
#[derive(Debug, Clone)]
enum Token {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Quote,
    Backtick,
    Tilde,
    TildeAt,
    At,
    Caret,
    Amp,
    String(String),
    Number(i64),
    Symbol(String),
    Keyword(String),
}

// Tokenizer
struct Tokenizer<'a> {
    input: Chars<'a>,
    current: Option<char>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Tokenizer {
            input: chars,
            current,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let result = self.current;
        self.current = self.input.next();
        result
    }

    fn peek_char(&self) -> Option<char> {
        self.current
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if !c.is_whitespace() && c != ',' {
                break;
            }
            self.next_char();
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        let mut result = String::new();
        let mut escaped = false;

        while let Some(c) = self.next_char() {
            if escaped {
                match c {
                    'n' => result.push('\n'),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", c)),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Ok(result);
            } else {
                result.push(c);
            }
        }

        Err("Unterminated string".to_string())
    }

    fn read_atom(&mut self, first_char: char) -> Token {
        let mut atom = String::new();
        atom.push(first_char);

        while let Some(c) = self.peek_char() {
            if "()[]{}'\"`~^@,".contains(c) || c.is_whitespace() {
                break;
            }
            atom.push(self.next_char().unwrap());
        }

        // Try to parse as number first
        if let Ok(n) = atom.parse::<i64>() {
            return Token::Number(n);
        }

        // Check for keyword
        if atom.starts_with(':') {
            return Token::Keyword(atom[1..].to_string());
        }

        // Otherwise it's a symbol
        Token::Symbol(atom)
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        self.consume_whitespace();

        match self.next_char() {
            None => Ok(None),
            Some(c) => {
                match c {
                    '(' => Ok(Some(Token::LeftParen)),
                    ')' => Ok(Some(Token::RightParen)),
                    '[' => Ok(Some(Token::LeftBracket)),
                    ']' => Ok(Some(Token::RightBracket)),
                    '\'' => Ok(Some(Token::Quote)),
                    '`' => Ok(Some(Token::Backtick)),
                    '~' => {
                        if let Some('@') = self.peek_char() {
                            self.next_char();
                            Ok(Some(Token::TildeAt))
                        } else {
                            Ok(Some(Token::Tilde))
                        }
                    }
                    '@' => Ok(Some(Token::At)),
                    '^' => Ok(Some(Token::Caret)),
                    '&' => Ok(Some(Token::Amp)),
                    '"' => Ok(Some(Token::String(self.read_string()?))),
                    ';' => {
                        // Skip comment
                        while let Some(c) = self.next_char() {
                            if c == '\n' {
                                break;
                            }
                        }
                        self.next_token()
                    }
                    _ => Ok(Some(self.read_atom(c))),
                }
            }
        }
    }
}

// Reader
pub fn read_str(input: &str) -> Result<MalType, String> {
    let mut tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty input".to_string());
    }
    read_form(&mut tokens)
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokenizer = Tokenizer::new(input);
    let mut tokens = Vec::new();

    while let Some(token) = tokenizer.next_token()? {
        tokens.push(token);
    }

    Ok(tokens)
}

fn read_form(tokens: &mut Vec<Token>) -> Result<MalType, String> {
    if tokens.is_empty() {
        return Err("Unexpected EOF".to_string());
    }

    match tokens.remove(0) {
        Token::LeftParen => read_list(tokens),
        Token::LeftBracket => read_vector(tokens),
        Token::Quote => {
            let form = read_form(tokens)?;
            Ok(MalType::List(vec![
                MalType::Symbol("quote".to_string()),
                form,
            ]))
        }
        Token::Backtick => {
            let form = read_form(tokens)?;
            Ok(MalType::List(vec![
                MalType::Symbol("quasiquote".to_string()),
                form,
            ]))
        }
        Token::Tilde => {
            let form = read_form(tokens)?;
            Ok(MalType::List(vec![
                MalType::Symbol("unquote".to_string()),
                form,
            ]))
        }
        Token::TildeAt => {
            let form = read_form(tokens)?;
            Ok(MalType::List(vec![
                MalType::Symbol("splice-unquote".to_string()),
                form,
            ]))
        }
        Token::At => {
            let form = read_form(tokens)?;
            Ok(MalType::List(vec![
                MalType::Symbol("deref".to_string()),
                form,
            ]))
        }
        Token::Caret => {
            let meta = read_form(tokens)?;
            let form = read_form(tokens)?;
            Ok(MalType::List(vec![
                MalType::Symbol("with-meta".to_string()),
                form,
                meta,
            ]))
        }
        Token::Amp => Ok(MalType::Symbol("&".to_string())),
        Token::RightParen => Err("Unexpected ')'".to_string()),
        Token::RightBracket => Err("Unexpected ']'".to_string()),
        Token::String(s) => Ok(MalType::String(s)),
        Token::Number(n) => Ok(MalType::Number(n)),
        Token::Symbol(s) => Ok(MalType::Symbol(s)),
        Token::Keyword(s) => Ok(MalType::Keyword(s)),
    }
}

fn read_list(tokens: &mut Vec<Token>) -> Result<MalType, String> {
    let mut items = Vec::new();

    loop {
        if tokens.is_empty() {
            return Err("Expected ')', got EOF".to_string());
        }

        if let Some(Token::RightParen) = tokens.first() {
            tokens.remove(0);
            return Ok(MalType::List(items));
        }

        items.push(read_form(tokens)?);
    }
}

fn read_vector(tokens: &mut Vec<Token>) -> Result<MalType, String> {
    let mut items = Vec::new();

    loop {
        if tokens.is_empty() {
            return Err("Expected ']', got EOF".to_string());
        }

        if let Some(Token::RightBracket) = tokens.first() {
            tokens.remove(0);
            return Ok(MalType::Vector(items));
        }

        items.push(read_form(tokens)?);
    }
} 