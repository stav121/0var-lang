//! Lexical analyzer for the zvar language

pub mod token;

use crate::error::ZvarError;
use crate::span::Span;
use token::Token;

pub struct Lexer<'a> {
    input: &'a str,
    position: usize, // Current position in input
    current_char: Option<char>,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
            line: 1,
            column: 1,
        };
        lexer.current_char = lexer.input.chars().next();
        lexer
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
    }

    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    /// Skip whitespace (except newlines, which we track)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read a number literal
    fn read_number(&mut self) -> Result<i64, ZvarError> {
        let start_pos = self.position;
        let start_col = self.column;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let number_str = &self.input[start_pos..self.position];
        number_str.parse().map_err(|_| ZvarError::InvalidNumber {
            span: Span::new(self.line, start_col, self.line, self.column - 1),
            value: number_str.to_string(),
        })
    }

    /// Read a string literal
    fn read_string_literal(&mut self) -> Result<String, ZvarError> {
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // Skip opening quote
        let start_pos = self.position;

        while let Some(ch) = self.current_char {
            if ch == '"' {
                let content = self.input[start_pos..self.position].to_string();
                self.advance(); // Skip closing quote
                return Ok(content);
            } else if ch == '\n' {
                return Err(ZvarError::UnexpectedToken {
                    span: Span::new(start_line, start_col, self.line, self.column),
                    expected: "closing quote before newline".to_string(),
                    found: "newline".to_string(),
                });
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance(); // Skip backslash
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' | 't' | 'r' | '\\' | '"' => {
                            self.advance(); // Skip escaped character
                        }
                        _ => {
                            return Err(ZvarError::UnexpectedCharacter {
                                span: Span::new(self.line, self.column, self.line, self.column),
                                character: escaped,
                            });
                        }
                    }
                } else {
                    return Err(ZvarError::UnexpectedToken {
                        span: Span::new(start_line, start_col, self.line, self.column),
                        expected: "escaped character".to_string(),
                        found: "end of file".to_string(),
                    });
                }
            } else {
                self.advance();
            }
        }

        Err(ZvarError::UnexpectedToken {
            span: Span::new(start_line, start_col, self.line, self.column),
            expected: "closing quote".to_string(),
            found: "end of file".to_string(),
        })
    }

    /// Read an identifier or entity (v$0, c$1, f$2, etc.)
    fn read_identifier(&mut self) -> Result<Token, ZvarError> {
        let start_pos = self.position;
        let start_col = self.column;

        // Read the identifier
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                self.advance();
            } else {
                break;
            }
        }

        let identifier = &self.input[start_pos..self.position];

        // Check if it's an entity (v$N, c$N, f$N)
        if let Some(token) = self.parse_entity(identifier)? {
            return Ok(token);
        }

        // Check for keywords
        let token = match identifier {
            "fn" => Token::Fn,
            "main" => Token::Main,
            "ret" => Token::Ret,
            "int" => Token::Int,
            "str" => Token::Str,
            "bool" => Token::Bool,
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "describe" => Token::Describe,
            "print" => Token::Print,
            _ => {
                return Err(ZvarError::UnknownIdentifier {
                    span: Span::new(self.line, start_col, self.line, self.column - 1),
                    name: identifier.to_string(),
                });
            }
        };

        Ok(token)
    }

    /// Parse entity tokens (v$N, c$N, f$N)
    fn parse_entity(&self, identifier: &str) -> Result<Option<Token>, ZvarError> {
        if identifier.len() < 3 {
            return Ok(None);
        }

        let prefix = &identifier[0..1];
        if &identifier[1..2] != "$" {
            return Ok(None);
        }

        let number_str = &identifier[2..];
        let number: u32 = number_str
            .parse()
            .map_err(|_| ZvarError::InvalidEntityNumber {
                span: Span::new(
                    self.line,
                    self.column - identifier.len() as u32,
                    self.line,
                    self.column - 1,
                ),
                entity: identifier.to_string(),
            })?;

        let token = match prefix {
            "v" => Token::Variable(number),
            "c" => Token::Constant(number),
            "f" => Token::Function(number),
            _ => return Ok(None),
        };

        Ok(Some(token))
    }

    /// Read a documentation comment (///)
    fn read_doc_comment(&mut self) -> Token {
        // Skip the ///
        self.advance(); // /
        self.advance(); // /
        self.advance(); // /

        // Skip any immediate whitespace
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }

        // Read until end of line
        let start_pos = self.position;
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }

        let comment = self.input[start_pos..self.position].to_string();
        Token::DocComment(comment)
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token, ZvarError> {
        loop {
            match self.current_char {
                None => return Ok(Token::Eof),

                Some(ch) if ch.is_whitespace() => {
                    if ch == '\n' {
                        self.advance();
                        return Ok(Token::Newline);
                    } else {
                        self.skip_whitespace();
                    }
                }

                Some(ch) if ch.is_ascii_digit() => {
                    let number = self.read_number()?;
                    return Ok(Token::Integer(number));
                }

                Some('=') => {
                    if self.peek() == Some('=') {
                        self.advance(); // =
                        self.advance(); // =
                        return Ok(Token::Equal);
                    } else {
                        self.advance();
                        return Ok(Token::Assign);
                    }
                }

                Some('!') => {
                    if self.peek() == Some('=') {
                        self.advance(); // !
                        self.advance(); // =
                        return Ok(Token::NotEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Not);
                    }
                }

                Some('<') => {
                    if self.peek() == Some('=') {
                        self.advance(); // <
                        self.advance(); // =
                        return Ok(Token::LessEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Less);
                    }
                }

                Some('>') => {
                    if self.peek() == Some('=') {
                        self.advance(); // >
                        self.advance(); // =
                        return Ok(Token::GreaterEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Greater);
                    }
                }

                Some('&') => {
                    if self.peek() == Some('&') {
                        self.advance(); // &
                        self.advance(); // &
                        return Ok(Token::And);
                    } else {
                        return Err(ZvarError::UnexpectedCharacter {
                            span: Span::new(self.line, self.column, self.line, self.column),
                            character: '&',
                        });
                    }
                }

                Some('|') => {
                    if self.peek() == Some('|') {
                        self.advance(); // |
                        self.advance(); // |
                        return Ok(Token::Or);
                    } else {
                        return Err(ZvarError::UnexpectedCharacter {
                            span: Span::new(self.line, self.column, self.line, self.column),
                            character: '|',
                        });
                    }
                }
                Some('"') => {
                    let string_literal = self.read_string_literal()?;
                    return Ok(Token::String(string_literal));
                }

                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    return self.read_identifier();
                }

                Some('/') => {
                    if self.peek() == Some('/') {
                        self.advance(); // First /
                        if self.peek() == Some('/') {
                            // Documentation comment ///
                            return Ok(self.read_doc_comment());
                        } else {
                            // Regular comment //, skip to end of line
                            while let Some(ch) = self.current_char {
                                if ch == '\n' {
                                    break;
                                }
                                self.advance();
                            }
                        }
                    } else {
                        self.advance();
                        return Ok(Token::Divide);
                    }
                }

                Some('+') => {
                    self.advance();
                    return Ok(Token::Plus);
                }
                Some('-') => {
                    if self.peek() == Some('>') {
                        self.advance(); // -
                        self.advance(); // >
                        return Ok(Token::Arrow);
                    } else {
                        self.advance();
                        return Ok(Token::Minus);
                    }
                }
                Some('*') => {
                    self.advance();
                    return Ok(Token::Multiply);
                }
                Some('=') => {
                    self.advance();
                    return Ok(Token::Assign);
                }
                Some('(') => {
                    self.advance();
                    return Ok(Token::LeftParen);
                }
                Some(')') => {
                    self.advance();
                    return Ok(Token::RightParen);
                }
                Some('{') => {
                    self.advance();
                    return Ok(Token::LeftBrace);
                }
                Some('}') => {
                    self.advance();
                    return Ok(Token::RightBrace);
                }
                Some(';') => {
                    self.advance();
                    return Ok(Token::Semicolon);
                }
                Some(',') => {
                    self.advance();
                    return Ok(Token::Comma);
                }

                Some(ch) => {
                    return Err(ZvarError::UnexpectedCharacter {
                        span: Span::new(self.line, self.column, self.line, self.column),
                        character: ch,
                    });
                }
            }
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ZvarError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("+ - * / = ( ) { } ; ,");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Multiply);
        assert_eq!(tokens[3], Token::Divide);
        assert_eq!(tokens[4], Token::Assign);
    }

    #[test]
    fn test_entities() {
        let mut lexer = Lexer::new("v$0 c$1 f$2");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Variable(0));
        assert_eq!(tokens[1], Token::Constant(1));
        assert_eq!(tokens[2], Token::Function(2));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("fn main ret int describe print");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Main);
        assert_eq!(tokens[2], Token::Ret);
        assert_eq!(tokens[3], Token::Int);
        assert_eq!(tokens[4], Token::Describe);
        assert_eq!(tokens[5], Token::Print);
    }

    #[test]
    fn test_string_literals() {
        let mut lexer = Lexer::new(r#""hello world" "test""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        assert_eq!(tokens[1], Token::String("test".to_string()));
    }

    #[test]
    fn test_string_with_spaces() {
        let mut lexer = Lexer::new(r#""hello world with spaces""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[0],
            Token::String("hello world with spaces".to_string())
        );
    }

    #[test]
    fn test_empty_string() {
        let mut lexer = Lexer::new(r#""""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::String("".to_string()));
    }
}
