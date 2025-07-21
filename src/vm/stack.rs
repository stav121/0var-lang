//! Stack implementation for the zvar virtual machine

use crate::{
    error::{ZvarError, ZvarResult},
    vm::value::Value,
};

/// Stack size limit to prevent stack overflow
const STACK_SIZE_LIMIT: usize = 1024;

/// Runtime stack for the virtual machine
#[derive(Debug)]
pub struct Stack {
    values: Vec<Value>,
    max_size: usize,
}

impl Stack {
    /// Create a new stack with default size limit
    pub fn new() -> Self {
        Stack {
            values: Vec::new(),
            max_size: STACK_SIZE_LIMIT,
        }
    }

    /// Create a new stack with custom size limit
    pub fn with_capacity(max_size: usize) -> Self {
        Stack {
            values: Vec::with_capacity(max_size.min(STACK_SIZE_LIMIT)),
            max_size,
        }
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) -> ZvarResult<()> {
        if self.values.len() >= self.max_size {
            return Err(ZvarError::StackOverflow);
        }

        self.values.push(value);
        Ok(())
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> ZvarResult<Value> {
        self.values.pop().ok_or(ZvarError::StackUnderflow)
    }

    /// Peek at the top value without removing it
    pub fn peek(&self) -> ZvarResult<&Value> {
        self.values.last().ok_or(ZvarError::StackUnderflow)
    }

    /// Peek at the top value mutably
    pub fn peek_mut(&mut self) -> ZvarResult<&mut Value> {
        self.values.last_mut().ok_or(ZvarError::StackUnderflow)
    }

    /// Duplicate the top value
    pub fn dup(&mut self) -> ZvarResult<()> {
        let top = self.peek()?.clone();
        self.push(top)
    }

    /// Get the current stack size
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Get a value at a specific depth (0 = top, 1 = second from top, etc.)
    pub fn get(&self, depth: usize) -> ZvarResult<&Value> {
        if depth >= self.values.len() {
            return Err(ZvarError::StackUnderflow);
        }

        let index = self.values.len() - 1 - depth;
        Ok(&self.values[index])
    }

    /// Set a value at a specific depth
    pub fn set(&mut self, depth: usize, value: Value) -> ZvarResult<()> {
        if depth >= self.values.len() {
            return Err(ZvarError::StackUnderflow);
        }

        let index = self.values.len() - 1 - depth;
        self.values[index] = value;
        Ok(())
    }

    /// Get the maximum stack size reached (for debugging)
    pub fn high_water_mark(&self) -> usize {
        // In a more sophisticated implementation, we'd track this
        self.values.len()
    }

    /// Print the stack contents (for debugging)
    pub fn debug_print(&self) {
        println!("Stack (size: {}):", self.values.len());
        for (i, value) in self.values.iter().rev().enumerate() {
            let marker = if i == 0 { " -> " } else { "    " };
            println!("{}{}: {}", marker, self.values.len() - 1 - i, value);
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_stack_operations() {
        let mut stack = Stack::new();

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);

        // Push values
        stack.push(Value::Int(10)).unwrap();
        stack.push(Value::Int(20)).unwrap();

        assert_eq!(stack.len(), 2);
        assert!(!stack.is_empty());

        // Peek
        assert_eq!(stack.peek().unwrap(), &Value::Int(20));
        assert_eq!(stack.len(), 2); // Peek doesn't remove

        // Pop values
        assert_eq!(stack.pop().unwrap(), Value::Int(20));
        assert_eq!(stack.pop().unwrap(), Value::Int(10));

        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_underflow() {
        let mut stack = Stack::new();

        let result = stack.pop();
        assert!(matches!(result, Err(ZvarError::StackUnderflow)));

        let result = stack.peek();
        assert!(matches!(result, Err(ZvarError::StackUnderflow)));
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = Stack::with_capacity(2);

        stack.push(Value::Int(1)).unwrap();
        stack.push(Value::Int(2)).unwrap();

        let result = stack.push(Value::Int(3));
        assert!(matches!(result, Err(ZvarError::StackOverflow)));
    }

    #[test]
    fn test_duplicate() {
        let mut stack = Stack::new();
        stack.push(Value::Int(42)).unwrap();

        stack.dup().unwrap();

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop().unwrap(), Value::Int(42));
        assert_eq!(stack.pop().unwrap(), Value::Int(42));
    }

    #[test]
    fn test_indexed_access() {
        let mut stack = Stack::new();
        stack.push(Value::Int(10)).unwrap();
        stack.push(Value::Int(20)).unwrap();
        stack.push(Value::Int(30)).unwrap();

        // Top of stack is depth 0
        assert_eq!(stack.get(0).unwrap(), &Value::Int(30));
        assert_eq!(stack.get(1).unwrap(), &Value::Int(20));
        assert_eq!(stack.get(2).unwrap(), &Value::Int(10));

        // Modify value at depth
        stack.set(1, Value::Int(99)).unwrap();
        assert_eq!(stack.get(1).unwrap(), &Value::Int(99));
    }
}
