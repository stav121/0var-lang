//! Virtual machine for executing zvar bytecode

pub mod builtins;
pub mod stack;
pub mod value;

use crate::{
    codegen::{
        debug_info::DebugInfo,
        instruction::{Bytecode, Instruction},
    },
    error::{ZvarError, ZvarResult},
};

use builtins::Builtins;
use stack::Stack;
use std::collections::HashMap;
use value::Value;

/// Virtual machine state
#[derive(Debug)]
pub struct VM {
    /// Runtime stack
    stack: Stack,
    /// Variable storage (indexed by slot number)
    variables: Vec<Option<Value>>,
    /// Built-in functions
    builtins: Builtins,
    /// Function call stack for tracking returns
    call_stack: Vec<CallFrame>,
    /// Current instruction pointer
    ip: usize,
    /// Currently executing bytecode
    bytecode: Option<Bytecode>,
    /// Debug information
    debug_info: Option<DebugInfo>,
    /// Entity documentation (for runtime describe() calls)
    entity_docs: HashMap<String, String>,
    /// Debug mode flag
    debug_mode: bool,
}

/// Call frame for function calls
#[derive(Debug, Clone)]
struct CallFrame {
    return_address: usize,
    function_name: String,
    saved_variables: Vec<Option<Value>>,
    variable_base: usize,
}

impl VM {
    /// Create a new virtual machine
    pub fn new() -> Self {
        VM {
            stack: Stack::new(),
            variables: Vec::new(),
            builtins: Builtins::new(),
            call_stack: Vec::new(),
            ip: 0,
            bytecode: None,
            debug_info: None,
            entity_docs: HashMap::new(),
            debug_mode: false,
        }
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }

    /// Debug method to show stack state
    pub fn debug_stack_state(&self, instruction: &str) {
        let stack_preview = if self.stack.len() > 0 {
            let items: Vec<String> = (0..self.stack.len().min(5))
                .map(|i| match self.stack.get(self.stack.len() - 1 - i) {
                    Ok(val) => format!("{}", val),
                    Err(_) => "?".to_string(),
                })
                .collect();
            format!(
                "[{}] (top->bottom, {} total)",
                items.join(", "),
                self.stack.len()
            )
        } else {
            "[empty]".to_string()
        };

        if self.debug_mode {
            println!(
                "DEBUG: {} - Stack: {}, IP: {}",
                instruction, stack_preview, self.ip
            );
        }
    }

    /// Load bytecode and debug info into the VM
    pub fn load(&mut self, bytecode: Bytecode, debug_info: Option<DebugInfo>) {
        // Calculate required variable slots
        let max_var_slot = bytecode
            .instructions
            .iter()
            .filter_map(|inst| match inst {
                Instruction::LoadVar(slot) | Instruction::StoreVar(slot) => Some(*slot),
                _ => None,
            })
            .max()
            .unwrap_or(0);

        // Initialize variable storage
        self.variables = vec![None; (max_var_slot + 1) as usize];

        // Set entry point
        self.ip = bytecode.entry_point;

        // Load debug info
        if let Some(debug) = &debug_info {
            for (entity, doc) in &debug.entity_docs {
                self.entity_docs.insert(entity.clone(), doc.clone());
            }
        }

        self.bytecode = Some(bytecode);
        self.debug_info = debug_info;
    }

    /// Execute the loaded bytecode
    pub fn run(&mut self) -> ZvarResult<()> {
        loop {
            // Check if we're at the end or past the end
            let instruction_count = self
                .bytecode
                .as_ref()
                .ok_or_else(|| ZvarError::runtime("No bytecode loaded"))?
                .instructions
                .len();

            if self.ip >= instruction_count {
                break;
            }

            // Clone the instruction to avoid borrowing issues
            let instruction = self.bytecode.as_ref().unwrap().instructions[self.ip].clone();

            // DEBUG: Show state before execution
            if self.debug_mode {
                println!("DEBUG: Before executing {} at IP {}", instruction, self.ip);
                self.debug_stack_state("BEFORE");
            }

            match self.execute_instruction(&instruction)? {
                ExecutionResult::Continue => {
                    self.ip += 1;
                }
                ExecutionResult::Jump(new_ip) => {
                    if self.debug_mode {
                        println!("DEBUG: Jumping from {} to {}", self.ip, new_ip);
                    }
                    self.ip = new_ip;
                }
                ExecutionResult::Return => {
                    if self.debug_mode {
                        println!("DEBUG: Function return triggered");
                        self.debug_stack_state("BEFORE RETURN");
                    }
                    if let Some(frame) = self.call_stack.pop() {
                        // Save return value BEFORE restoring variables
                        let return_value = if !self.stack.is_empty() {
                            let val = self.stack.pop()?;
                            if self.debug_mode {
                                println!("DEBUG: Saved return value: {}", val);
                            }
                            Some(val)
                        } else {
                            if self.debug_mode {
                                println!("DEBUG: No return value on stack");
                            }
                            None
                        };

                        // Restore the saved variables
                        if self.debug_mode {
                            println!(
                                "DEBUG: Restoring {} saved variables",
                                frame.saved_variables.len()
                            );
                        }
                        for (i, saved_var) in frame.saved_variables.iter().enumerate() {
                            if i < self.variables.len() {
                                self.variables[i] = saved_var.clone();
                            }
                        }

                        // Put return value back AFTER restoring variables
                        if let Some(value) = return_value {
                            self.stack.push(value.clone())?;
                            if self.debug_mode {
                                println!("DEBUG: Restored return value to stack: {}", value);
                            }
                        }

                        if self.debug_mode {
                            println!("DEBUG: Returning to IP {}", frame.return_address);
                        }
                        self.ip = frame.return_address;
                    } else {
                        // Return from main, halt execution
                        if self.debug_mode {
                            println!("DEBUG: Main function return - halting");
                        }
                        break;
                    }
                }
                ExecutionResult::Halt => {
                    if self.debug_mode {
                        println!("DEBUG: HALT instruction - stopping execution");
                    }
                    break;
                }
            }

            if self.debug_mode {
                // DEBUG: Show state after execution
                self.debug_stack_state("AFTER");
                println!("DEBUG: ----------------------------------------");
            }
        }

        Ok(())
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &Instruction) -> ZvarResult<ExecutionResult> {
        // Add debug information for stack underflow issues
        match instruction {
            Instruction::Pop => {
                if self.stack.is_empty() {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: tried to POP from empty stack at IP {}",
                        self.ip
                    )));
                }
                self.stack.pop()?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Push(value) => {
                self.stack.push(value.clone().into())?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Dup => {
                self.stack.dup()?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Add => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: ADD needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.add(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Sub => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: SUB needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.sub(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Mul => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: MUL needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.mul(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Div => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: DIV needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.div(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            // NEW: Comparison operations
            Instruction::Equal => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: EQUAL needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.equal(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::NotEqual => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: NOT_EQUAL needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.not_equal(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Less => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: LESS needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.less(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Greater => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: GREATER needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.greater(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::LessEqual => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: LESS_EQUAL needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.less_equal(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::GreaterEqual => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: GREATER_EQUAL needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.greater_equal(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            // NEW: Logical operations
            Instruction::And => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: AND needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.logical_and(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Or => {
                if self.stack.len() < 2 {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: OR needs 2 values, only {} available at IP {}",
                        self.stack.len(),
                        self.ip
                    )));
                }
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.logical_or(&b)?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Not => {
                if self.stack.is_empty() {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: NOT needs 1 value, stack is empty at IP {}",
                        self.ip
                    )));
                }
                let a = self.stack.pop()?;
                let result = a.logical_not()?;
                self.stack.push(result)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::LoadVar(slot) => {
                if *slot as usize >= self.variables.len() {
                    return Err(ZvarError::runtime(format!(
                        "Invalid variable slot: {}",
                        slot
                    )));
                }

                let value = self.variables[*slot as usize].clone().ok_or_else(|| {
                    ZvarError::runtime(format!("Uninitialized variable v${}", slot))
                })?;

                self.stack.push(value)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::StoreVar(slot) => {
                if self.stack.is_empty() {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: STOREVAR needs 1 value, stack is empty at IP {}",
                        self.ip
                    )));
                }
                if *slot as usize >= self.variables.len() {
                    return Err(ZvarError::runtime(format!(
                        "Invalid variable slot: {}",
                        slot
                    )));
                }

                let value = self.stack.pop()?;
                self.variables[*slot as usize] = Some(value);
                Ok(ExecutionResult::Continue)
            }

            Instruction::LoadConst(index) => {
                let bytecode = self.bytecode.as_ref().unwrap();
                let value = bytecode.get_constant(*index).ok_or_else(|| {
                    ZvarError::runtime(format!("Invalid constant index: {}", index))
                })?;

                self.stack.push(value.clone().into())?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Call(name, argc) => {
                if self.builtins.is_builtin(name) {
                    // Built-in function call
                    self.builtins.call(name, &mut self.stack)?;
                    Ok(ExecutionResult::Continue)
                } else {
                    // User-defined function call
                    if let Some(debug) = &self.debug_info {
                        if let Some(func_start) = debug.get_function_start(name) {
                            // Save the current values of variables that will be overwritten
                            let mut saved_vars = Vec::new();
                            for i in 0..*argc {
                                if (i as usize) < self.variables.len() {
                                    saved_vars.push(self.variables[i as usize].clone());
                                } else {
                                    saved_vars.push(None);
                                }
                            }

                            // Ensure we have enough variable slots
                            if (*argc as usize) > self.variables.len() {
                                self.variables.resize(*argc as usize, None);
                            }

                            // Store function arguments into parameter variables (v$0, v$1, etc.)
                            let mut args = Vec::new();
                            for _ in 0..*argc {
                                args.push(self.stack.pop()?);
                            }
                            args.reverse(); // Put them in correct order

                            // Store each argument in slots 0, 1, 2, etc.
                            for (i, arg) in args.iter().enumerate() {
                                self.variables[i] = Some(arg.clone());
                            }

                            // Push call frame with saved variables
                            // FIX: Set return address to current IP + 1 (the instruction after CALL)
                            self.call_stack.push(CallFrame {
                                return_address: self.ip + 1,
                                function_name: name.clone(),
                                saved_variables: saved_vars,
                                variable_base: 0,
                            });

                            // Jump to function
                            Ok(ExecutionResult::Jump(func_start))
                        } else {
                            Err(ZvarError::runtime(format!("Unknown function: {}", name)))
                        }
                    } else {
                        Err(ZvarError::runtime(
                            "No debug info available for function calls",
                        ))
                    }
                }
            }

            Instruction::Return | Instruction::ReturnValue => {
                // Let the main execution loop handle the return logic
                Ok(ExecutionResult::Return)
            }

            Instruction::Jump(address) => Ok(ExecutionResult::Jump(*address)),

            Instruction::JumpIfFalse(address) => {
                if self.stack.is_empty() {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: JUMP_IF_FALSE needs 1 value, stack is empty at IP {}",
                        self.ip
                    )));
                }
                let value = self.stack.pop()?;
                if !value.is_truthy() {
                    Ok(ExecutionResult::Jump(*address))
                } else {
                    Ok(ExecutionResult::Continue)
                }
            }

            Instruction::Print => {
                if self.stack.is_empty() {
                    return Err(ZvarError::runtime(format!(
                        "Stack underflow: PRINT needs 1 value, stack is empty at IP {}",
                        self.ip
                    )));
                }
                self.builtins.call("print", &mut self.stack)?;
                Ok(ExecutionResult::Continue)
            }

            Instruction::Describe(entity, description) => {
                // Store documentation for runtime access
                self.entity_docs.insert(entity.clone(), description.clone());
                if self.debug_mode {
                    println!("Debug: {} - {}", entity, description);
                }
                Ok(ExecutionResult::Continue)
            }

            Instruction::Halt => Ok(ExecutionResult::Halt),

            Instruction::Nop => Ok(ExecutionResult::Continue),
        }
    }

    /// Get current stack state (for debugging)
    pub fn debug_stack(&self) {
        self.stack.debug_print();
    }

    /// Get variable state (for debugging)
    pub fn debug_variables(&self) {
        println!("Variables:");
        for (i, var) in self.variables.iter().enumerate() {
            match var {
                Some(value) => println!("  v${}: {}", i, value),
                None => println!("  v${}: <uninitialized>", i),
            }
        }
    }

    /// Get entity documentation
    pub fn get_entity_doc(&self, entity: &str) -> Option<&String> {
        self.entity_docs.get(entity)
    }

    /// Reset the VM state
    pub fn reset(&mut self) {
        self.stack.clear();
        self.variables.clear();
        self.call_stack.clear();
        self.ip = 0;
        self.entity_docs.clear();
    }
}

/// Result of executing an instruction
#[derive(Debug, PartialEq)]
enum ExecutionResult {
    Continue,    // Continue to next instruction
    Jump(usize), // Jump to specific instruction
    Return,      // Return from function
    Halt,        // Stop execution
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::instruction::{Bytecode, Instruction, Value as InstValue};

    #[test]
    fn test_basic_arithmetic() {
        let mut vm = VM::new();
        let mut bytecode = Bytecode::new();

        // Program: 5 + 3
        bytecode.emit(Instruction::Push(InstValue::Int(5)));
        bytecode.emit(Instruction::Push(InstValue::Int(3)));
        bytecode.emit(Instruction::Add);
        bytecode.emit(Instruction::Halt);

        vm.load(bytecode, None);
        vm.run().unwrap();

        // Result should be on stack
        assert_eq!(vm.stack.pop().unwrap(), Value::Int(8));
    }

    #[test]
    fn test_variable_operations() {
        let mut vm = VM::new();
        let mut bytecode = Bytecode::new();

        // Program: v$0 = 42; load v$0
        bytecode.emit(Instruction::Push(InstValue::Int(42)));
        bytecode.emit(Instruction::StoreVar(0));
        bytecode.emit(Instruction::LoadVar(0));
        bytecode.emit(Instruction::Halt);

        vm.load(bytecode, None);
        vm.run().unwrap();

        // v$0 should contain 42
        assert_eq!(vm.stack.pop().unwrap(), Value::Int(42));
    }

    #[test]
    fn test_print_builtin() {
        let mut vm = VM::new();
        let mut bytecode = Bytecode::new();

        // Program: print(42)
        bytecode.emit(Instruction::Push(InstValue::Int(42)));
        bytecode.emit(Instruction::Print);
        bytecode.emit(Instruction::Halt);

        vm.load(bytecode, None);
        let result = vm.run();

        assert!(result.is_ok());
        assert!(vm.stack.is_empty()); // Print consumes the value
    }

    #[test]
    fn test_stack_underflow_error() {
        let mut vm = VM::new();
        let mut bytecode = Bytecode::new();

        // Program: try to pop from empty stack
        bytecode.emit(Instruction::Pop);

        vm.load(bytecode, None);
        let result = vm.run();

        assert!(matches!(result, Err(ZvarError::StackUnderflow)));
    }

    #[test]
    fn test_division_by_zero() {
        let mut vm = VM::new();
        let mut bytecode = Bytecode::new();

        // Program: 10 / 0
        bytecode.emit(Instruction::Push(InstValue::Int(10)));
        bytecode.emit(Instruction::Push(InstValue::Int(0)));
        bytecode.emit(Instruction::Div);

        vm.load(bytecode, None);
        let result = vm.run();

        assert!(matches!(result, Err(ZvarError::DivisionByZero { .. })));
    }
}
