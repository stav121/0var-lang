//! Symbol table for tracking entities and their metadata

use crate::{error::ZvarError, span::Span};
use std::collections::HashMap;

/// Type of entity in the symbol table
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Variable {
        value_type: ValueType,
    },
    Constant {
        value_type: ValueType,
    },
    Function {
        params: Vec<ValueType>,
        return_type: ValueType,
    },
}

/// Value types supported by the language
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Int,
    Str,
    Bool,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Int => write!(f, "int"),
            ValueType::Str => write!(f, "str"),
            ValueType::Bool => write!(f, "bool"),
        }
    }
}

/// Symbol information stored in the table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub entity_type: EntityType,
    pub definition_span: Span,
    pub documentation: Option<String>,
    pub is_initialized: bool,
}

impl Symbol {
    pub fn new(entity_type: EntityType, definition_span: Span) -> Self {
        Symbol {
            entity_type,
            definition_span,
            documentation: None,
            is_initialized: false,
        }
    }

    pub fn with_documentation(mut self, doc: String) -> Self {
        self.documentation = Some(doc);
        self
    }

    pub fn mark_initialized(mut self) -> Self {
        self.is_initialized = true;
        self
    }

    pub fn is_variable(&self) -> bool {
        matches!(self.entity_type, EntityType::Variable { .. })
    }

    pub fn is_constant(&self) -> bool {
        matches!(self.entity_type, EntityType::Constant { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self.entity_type, EntityType::Function { .. })
    }

    pub fn get_type(&self) -> Option<&ValueType> {
        match &self.entity_type {
            EntityType::Variable { value_type } => Some(value_type),
            EntityType::Constant { value_type } => Some(value_type),
            EntityType::Function { return_type, .. } => Some(return_type),
        }
    }
}

/// Symbol table with scope management
#[derive(Debug)]
pub struct SymbolTable {
    // Stack of scopes, each scope is a HashMap of entity names to symbols
    scopes: Vec<HashMap<String, Symbol>>,
    // Global documentation comments waiting to be attached
    pending_docs: Vec<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()], // Start with global scope
            pending_docs: Vec::new(),
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add pending documentation comment
    pub fn add_pending_doc(&mut self, doc: String) {
        self.pending_docs.push(doc);
    }

    /// Take all pending documentation and combine it
    pub fn take_pending_docs(&mut self) -> Option<String> {
        if self.pending_docs.is_empty() {
            None
        } else {
            let combined = self.pending_docs.join("\n");
            self.pending_docs.clear();
            Some(combined)
        }
    }

    /// Define a new symbol
    pub fn define(&mut self, name: String, mut symbol: Symbol) -> Result<(), ZvarError> {
        // Check if already defined in current scope
        if let Some(current_scope) = self.scopes.last() {
            if let Some(existing) = current_scope.get(&name) {
                return Err(ZvarError::EntityAlreadyDefined {
                    span: symbol.definition_span,
                    name,
                    previous_span: Some(existing.definition_span),
                });
            }
        }

        // Attach any pending documentation
        if let Some(docs) = self.take_pending_docs() {
            symbol.documentation = Some(docs);
        }

        // Add to current scope
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, symbol);
        }

        Ok(())
    }

    /// Look up a symbol in all scopes (starting from innermost)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Look up a symbol mutably
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Add documentation to an existing symbol
    pub fn add_documentation(&mut self, name: &str, doc: String) -> Result<(), ZvarError> {
        if let Some(symbol) = self.lookup_mut(name) {
            if let Some(existing_doc) = &symbol.documentation {
                symbol.documentation = Some(format!("{}\n{}", existing_doc, doc));
            } else {
                symbol.documentation = Some(doc);
            }
            Ok(())
        } else {
            Err(ZvarError::UndefinedEntity {
                span: Span::new(0, 0, 0, 0), // We don't have span info here
                name: name.to_string(),
            })
        }
    }

    /// Get all symbols in current scope (for debugging)
    pub fn current_scope_symbols(&self) -> Vec<(&String, &Symbol)> {
        if let Some(scope) = self.scopes.last() {
            scope.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all symbols across all scopes
    pub fn all_symbols(&self) -> Vec<(&String, &Symbol)> {
        let mut symbols = Vec::new();
        for scope in &self.scopes {
            for (name, symbol) in scope {
                symbols.push((name, symbol));
            }
        }
        symbols
    }

    /// Clear pending documentation (used when comments don't apply to anything)
    pub fn clear_pending_docs(&mut self) {
        self.pending_docs.clear();
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_basic_operations() {
        let mut table = SymbolTable::new();
        let span = Span::new(1, 1, 1, 5);

        // Define a variable
        let symbol = Symbol::new(
            EntityType::Variable {
                value_type: ValueType::Int,
            },
            span,
        );

        table.define("v$0".to_string(), symbol).unwrap();

        // Look it up
        let found = table.lookup("v$0").unwrap();
        assert!(found.is_variable());
        assert_eq!(found.definition_span, span);
    }

    #[test]
    fn test_scoping() {
        let mut table = SymbolTable::new();
        let span = Span::new(1, 1, 1, 5);

        // Define in global scope
        let symbol = Symbol::new(
            EntityType::Variable {
                value_type: ValueType::Int,
            },
            span,
        );
        table.define("v$0".to_string(), symbol).unwrap();

        // Enter new scope
        table.enter_scope();

        // Shadow in inner scope
        let symbol2 = Symbol::new(
            EntityType::Constant {
                value_type: ValueType::Int,
            },
            span,
        );
        table.define("v$0".to_string(), symbol2).unwrap();

        // Should find the inner scope version
        let found = table.lookup("v$0").unwrap();
        assert!(found.is_constant());

        // Exit scope
        table.exit_scope();

        // Should find the global version again
        let found = table.lookup("v$0").unwrap();
        assert!(found.is_variable());
    }

    #[test]
    fn test_pending_documentation() {
        let mut table = SymbolTable::new();
        let span = Span::new(1, 1, 1, 5);

        // Add pending docs
        table.add_pending_doc("First line".to_string());
        table.add_pending_doc("Second line".to_string());

        // Define symbol - should attach docs
        let symbol = Symbol::new(
            EntityType::Variable {
                value_type: ValueType::Int,
            },
            span,
        );
        table.define("v$0".to_string(), symbol).unwrap();

        // Check documentation was attached
        let found = table.lookup("v$0").unwrap();
        assert_eq!(
            found.documentation,
            Some("First line\nSecond line".to_string())
        );
    }

    #[test]
    fn test_duplicate_definition_error() {
        let mut table = SymbolTable::new();
        let span = Span::new(1, 1, 1, 5);

        let symbol1 = Symbol::new(
            EntityType::Variable {
                value_type: ValueType::Int,
            },
            span,
        );
        table.define("v$0".to_string(), symbol1).unwrap();

        let symbol2 = Symbol::new(
            EntityType::Constant {
                value_type: ValueType::Int,
            },
            span,
        );

        let result = table.define("v$0".to_string(), symbol2);
        assert!(matches!(
            result,
            Err(ZvarError::EntityAlreadyDefined { .. })
        ));
    }
}
