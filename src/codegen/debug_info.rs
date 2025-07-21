//! Debug information for bytecode

use crate::span::Span;
use std::collections::HashMap;

/// Debug information for a bytecode program
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Maps instruction index to source span
    pub instruction_spans: HashMap<usize, Span>,
    /// Maps entity names to their documentation
    pub entity_docs: HashMap<String, String>,
    /// Maps function names to their start instruction
    pub function_starts: HashMap<String, usize>,
    /// Original source code
    pub source: Option<String>,
}

impl DebugInfo {
    pub fn new() -> Self {
        DebugInfo {
            instruction_spans: HashMap::new(),
            entity_docs: HashMap::new(),
            function_starts: HashMap::new(),
            source: None,
        }
    }

    /// Set the original source code
    pub fn set_source(&mut self, source: String) {
        self.source = Some(source);
    }

    /// Add span information for an instruction
    pub fn add_instruction_span(&mut self, instruction_index: usize, span: Span) {
        self.instruction_spans.insert(instruction_index, span);
    }

    /// Add documentation for an entity
    pub fn add_entity_doc(&mut self, entity: String, doc: String) {
        self.entity_docs.insert(entity, doc);
    }

    /// Mark the start of a function
    pub fn mark_function_start(&mut self, name: String, instruction_index: usize) {
        self.function_starts.insert(name, instruction_index);
    }

    /// Get span for instruction
    pub fn get_instruction_span(&self, instruction_index: usize) -> Option<Span> {
        self.instruction_spans.get(&instruction_index).copied()
    }

    /// Get documentation for entity
    pub fn get_entity_doc(&self, entity: &str) -> Option<&String> {
        self.entity_docs.get(entity)
    }

    /// Get function start instruction
    pub fn get_function_start(&self, name: &str) -> Option<usize> {
        self.function_starts.get(name).copied()
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self::new()
    }
}
