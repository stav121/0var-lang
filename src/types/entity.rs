//! Entity type definitions for the zvar language

use crate::span::Span;
use std::fmt;

/// Entity types in the zvar language
#[derive(Debug, Clone, PartialEq)]
pub enum EntityKind {
    Variable,
    Constant,
    Function,
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityKind::Variable => write!(f, "variable"),
            EntityKind::Constant => write!(f, "constant"),
            EntityKind::Function => write!(f, "function"),
        }
    }
}

/// Entity reference with metadata
#[derive(Debug, Clone)]
pub struct EntityRef {
    pub kind: EntityKind,
    pub number: u32,
    pub span: Span,
}

impl EntityRef {
    pub fn new(kind: EntityKind, number: u32, span: Span) -> Self {
        EntityRef { kind, number, span }
    }

    pub fn variable(number: u32, span: Span) -> Self {
        EntityRef::new(EntityKind::Variable, number, span)
    }

    pub fn constant(number: u32, span: Span) -> Self {
        EntityRef::new(EntityKind::Constant, number, span)
    }

    pub fn function(number: u32, span: Span) -> Self {
        EntityRef::new(EntityKind::Function, number, span)
    }

    /// Get the full name (e.g., "v$0", "c$1", "f$2")
    pub fn full_name(&self) -> String {
        match self.kind {
            EntityKind::Variable => format!("v${}", self.number),
            EntityKind::Constant => format!("c${}", self.number),
            EntityKind::Function => format!("f${}", self.number),
        }
    }

    /// Get the prefix character
    pub fn prefix(&self) -> char {
        match self.kind {
            EntityKind::Variable => 'v',
            EntityKind::Constant => 'c',
            EntityKind::Function => 'f',
        }
    }
}

impl fmt::Display for EntityRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let span = Span::new(1, 1, 1, 5);

        let var = EntityRef::variable(0, span);
        assert_eq!(var.kind, EntityKind::Variable);
        assert_eq!(var.number, 0);
        assert_eq!(var.full_name(), "v$0");
        assert_eq!(var.prefix(), 'v');

        let const_ref = EntityRef::constant(1, span);
        assert_eq!(const_ref.full_name(), "c$1");

        let func = EntityRef::function(2, span);
        assert_eq!(func.full_name(), "f$2");
    }
}
