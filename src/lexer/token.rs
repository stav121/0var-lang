//! Token definitions for the zvar language

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    String(String),
    Boolean(bool), // true, false

    // Identifiers with prefixes
    Variable(u32), // v$0, v$1, etc.
    Constant(u32), // c$0, c$1, etc.
    Function(u32), // f$0, f$1, etc.

    // Keywords
    Fn,       // fn
    Main,     // main
    Ret,      // ret
    Int,      // int
    Str,      // str
    Bool,     // bool
    True,     // true
    False,    // false
    If,       // if
    Else,     // else
    Describe, // describe
    Print,    // print

    // Operators
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Assign,   // =

    // Comparison operators
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // Logical operators
    And, // &&
    Or,  // ||
    Not, // !

    // Delimiters
    LeftParen,  // (
    RightParen, // )
    LeftBrace,  // {
    RightBrace, // }
    Semicolon,  // ;
    Comma,      // ,
    Arrow,      // ->

    // Comments and Documentation
    DocComment(String), // /// comment

    // Special
    Eof,     // End of file
    Newline, // \n (for tracking lines)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::Variable(n) => write!(f, "v${}", n),
            Token::Constant(n) => write!(f, "c${}", n),
            Token::Function(n) => write!(f, "f${}", n),
            Token::Fn => write!(f, "fn"),
            Token::Main => write!(f, "main"),
            Token::Ret => write!(f, "ret"),
            Token::Int => write!(f, "int"),
            Token::Str => write!(f, "str"),
            Token::Bool => write!(f, "bool"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Describe => write!(f, "describe"),
            Token::Print => write!(f, "print"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Assign => write!(f, "="),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Arrow => write!(f, "->"),
            Token::DocComment(s) => write!(f, "/// {}", s),
            Token::Eof => write!(f, "EOF"),
            Token::Newline => write!(f, "\\n"),
        }
    }
}

impl Token {
    /// Check if this token is a documentation comment
    pub fn is_doc_comment(&self) -> bool {
        matches!(self, Token::DocComment(_))
    }

    /// Check if this token represents an entity (variable, constant, or function)
    pub fn is_entity(&self) -> bool {
        matches!(
            self,
            Token::Variable(_) | Token::Constant(_) | Token::Function(_)
        )
    }

    /// Get the number from an entity token (variable, constant, or function)
    pub fn entity_number(&self) -> Option<u32> {
        match self {
            Token::Variable(n) | Token::Constant(n) | Token::Function(n) => Some(*n),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(Token::Variable(0).to_string(), "v$0");
        assert_eq!(Token::Constant(1).to_string(), "c$1");
        assert_eq!(Token::Function(2).to_string(), "f$2");
        assert_eq!(Token::Integer(42).to_string(), "42");
        assert_eq!(Token::String("hello".to_string()).to_string(), "\"hello\"");
        assert_eq!(Token::Boolean(true).to_string(), "true");
        assert_eq!(Token::Boolean(false).to_string(), "false");
        assert_eq!(Token::Bool.to_string(), "bool");
        assert_eq!(Token::If.to_string(), "if");
        assert_eq!(Token::Equal.to_string(), "==");
        assert_eq!(Token::And.to_string(), "&&");
    }

    #[test]
    fn test_entity_methods() {
        assert!(Token::Variable(0).is_entity());
        assert!(Token::Constant(1).is_entity());
        assert!(Token::Function(2).is_entity());
        assert!(!Token::Int.is_entity());
        assert!(!Token::Bool.is_entity());
        assert!(!Token::String("test".to_string()).is_entity());

        assert_eq!(Token::Variable(5).entity_number(), Some(5));
        assert_eq!(Token::Int.entity_number(), None);
        assert_eq!(Token::Boolean(true).entity_number(), None);
    }
}
