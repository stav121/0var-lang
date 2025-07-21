//! Source code span tracking for error reporting and debugging

use std::fmt;

/// Represents a span of source code with line and column information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl Span {
    /// Create a new span
    pub fn new(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        Span {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    /// Create a span for a single character
    pub fn single(line: u32, column: u32) -> Self {
        Span::new(line, column, line, column)
    }

    /// Create a span from start to end
    pub fn from_to(start: Span, end: Span) -> Self {
        Span::new(
            start.start_line,
            start.start_column,
            end.end_line,
            end.end_column,
        )
    }

    /// Check if this span is on a single line
    pub fn is_single_line(&self) -> bool {
        self.start_line == self.end_line
    }

    /// Get the length of the span (only valid for single line spans)
    pub fn length(&self) -> Option<u32> {
        if self.is_single_line() {
            Some(self.end_column - self.start_column + 1)
        } else {
            None
        }
    }

    /// Check if this span contains a given position
    pub fn contains(&self, line: u32, column: u32) -> bool {
        if line < self.start_line || line > self.end_line {
            return false;
        }

        if line == self.start_line && column < self.start_column {
            return false;
        }

        if line == self.end_line && column > self.end_column {
            return false;
        }

        true
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_single_line() {
            if self.start_column == self.end_column {
                write!(f, "{}:{}", self.start_line, self.start_column)
            } else {
                write!(
                    f,
                    "{}:{}-{}",
                    self.start_line, self.start_column, self.end_column
                )
            }
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.start_line, self.start_column, self.end_line, self.end_column
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_column, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_column, 10);
    }

    #[test]
    fn test_single_line_span() {
        let span = Span::new(1, 5, 1, 10);
        assert!(span.is_single_line());
        assert_eq!(span.length(), Some(6));
    }

    #[test]
    fn test_multi_line_span() {
        let span = Span::new(1, 5, 3, 10);
        assert!(!span.is_single_line());
        assert_eq!(span.length(), None);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(2, 5, 2, 15);
        assert!(span.contains(2, 10));
        assert!(span.contains(2, 5));
        assert!(span.contains(2, 15));
        assert!(!span.contains(2, 4));
        assert!(!span.contains(2, 16));
        assert!(!span.contains(1, 10));
        assert!(!span.contains(3, 10));
    }

    #[test]
    fn test_span_display() {
        let single = Span::new(5, 10, 5, 15);
        assert_eq!(single.to_string(), "5:10-15");

        let point = Span::new(5, 10, 5, 10);
        assert_eq!(point.to_string(), "5:10");

        let multi = Span::new(2, 5, 4, 10);
        assert_eq!(multi.to_string(), "2:5-4:10");
    }
}
