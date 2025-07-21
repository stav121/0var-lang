//! Built-in functions for the zvar virtual machine

use crate::{
    error::{ZvarError, ZvarResult},
    vm::{stack::Stack, value::Value},
};
use std::collections::HashMap;

/// Type for built-in function implementations
pub type BuiltinFn = fn(&mut Stack) -> ZvarResult<()>;

/// Registry of built-in functions
#[derive(Debug)]
pub struct Builtins {
    functions: HashMap<String, BuiltinFn>,
}

impl Builtins {
    /// Create new builtins registry with default functions
    pub fn new() -> Self {
        let mut builtins = Builtins {
            functions: HashMap::new(),
        };

        // Register built-in functions
        builtins.register("print".to_string(), builtin_print);

        builtins
    }

    /// Register a built-in function
    pub fn register(&mut self, name: String, func: BuiltinFn) {
        self.functions.insert(name, func);
    }

    /// Call a built-in function
    pub fn call(&self, name: &str, stack: &mut Stack) -> ZvarResult<()> {
        if let Some(&func) = self.functions.get(name) {
            func(stack)
        } else {
            Err(ZvarError::runtime(format!(
                "Unknown built-in function: {}",
                name
            )))
        }
    }

    /// Check if a function is built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get list of all built-in function names
    pub fn function_names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in print function
/// Peeks at the top value and prints it without consuming it
fn builtin_print(stack: &mut Stack) -> ZvarResult<()> {
    let value = stack.peek()?;
    println!("{}", value);

    // Now pop the value since we've printed it
    stack.pop()?;
    Ok(())
}

// Future built-in functions can be added here:

/// Built-in debug function (prints stack state)
#[allow(dead_code)]
fn builtin_debug(stack: &mut Stack) -> ZvarResult<()> {
    stack.debug_print();
    Ok(())
}

/// Built-in typeof function (pushes type name as string)
#[allow(dead_code)]
fn builtin_typeof(stack: &mut Stack) -> ZvarResult<()> {
    let value = stack.pop()?;
    let type_name = value.type_name();

    // For now, we'll push it back as an integer representing the type
    // In the future, when we have strings, we'd push the type name as a string
    let type_id = match type_name {
        "int" => 1,
        _ => 0,
    };

    stack.push(Value::Int(type_id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry() {
        let builtins = Builtins::new();

        assert!(builtins.is_builtin("print"));
        assert!(!builtins.is_builtin("nonexistent"));

        let names = builtins.function_names();
        assert!(names.contains(&&"print".to_string()));
    }

    #[test]
    fn test_print_function() {
        let builtins = Builtins::new();
        let mut stack = Stack::new();

        stack.push(Value::Int(42)).unwrap();

        // This would print to stdout, but we can't easily test that
        // In a real implementation, we might want to inject an output writer
        let result = builtins.call("print", &mut stack);
        assert!(result.is_ok());
        assert!(stack.is_empty()); // Print should consume the value
    }

    #[test]
    fn test_unknown_function() {
        let builtins = Builtins::new();
        let mut stack = Stack::new();

        let result = builtins.call("unknown_func", &mut stack);
        assert!(matches!(result, Err(ZvarError::RuntimeError { .. })));
    }

    #[test]
    fn test_print_underflow() {
        let builtins = Builtins::new();
        let mut stack = Stack::new();

        // Try to print with empty stack
        let result = builtins.call("print", &mut stack);
        assert!(matches!(result, Err(ZvarError::StackUnderflow)));
    }
}
