//! Bytecode instruction set for the zvar virtual machine

use std::fmt;

/// Bytecode instructions for the zvar VM
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Stack operations
    Push(Value), // Push value onto stack
    Pop,         // Pop value from stack
    Dup,         // Duplicate top of stack

    // Arithmetic operations
    Add, // Pop two values, push sum
    Sub, // Pop two values, push difference (second - first)
    Mul, // Pop two values, push product
    Div, // Pop two values, push quotient (second / first)

    // Comparison operations
    Equal,        // Pop two values, push equality result
    NotEqual,     // Pop two values, push inequality result
    Less,         // Pop two values, push less-than result
    Greater,      // Pop two values, push greater-than result
    LessEqual,    // Pop two values, push less-equal result
    GreaterEqual, // Pop two values, push greater-equal result

    // Logical operations
    And, // Pop two values, push logical AND result
    Or,  // Pop two values, push logical OR result
    Not, // Pop one value, push logical NOT result

    // Variable operations
    LoadVar(u32),   // Load variable v$N onto stack
    StoreVar(u32),  // Store top of stack into variable v$N
    LoadConst(u32), // Load constant c$N onto stack

    // Function operations
    Call(String, u32), // Call function with N arguments
    Return,            // Return from function
    ReturnValue,       // Return with value from stack

    // Control flow
    Jump(usize),        // Unconditional jump to instruction
    JumpIfFalse(usize), // Jump if top of stack is false/zero

    // Built-in functions
    Print,                    // Print top of stack
    Describe(String, String), // Describe entity with documentation

    // Utility
    Halt, // Stop execution
    Nop,  // No operation
}

/// Runtime values that can be stored on the stack
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
}

impl Value {
    /// Get integer value, panic if not an integer
    pub fn as_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            Value::Str(_) => panic!("Expected integer, found string"),
            Value::Bool(_) => panic!("Expected integer, found boolean"),
        }
    }

    /// Get string value, panic if not a string
    pub fn as_str(&self) -> &str {
        match self {
            Value::Str(s) => s,
            Value::Int(_) => panic!("Expected string, found integer"),
            Value::Bool(_) => panic!("Expected string, found boolean"),
        }
    }

    /// Get boolean value, panic if not a boolean
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(_) => panic!("Expected boolean, found integer"),
            Value::Str(_) => panic!("Expected boolean, found string"),
        }
    }

    /// Check if value is truthy (non-zero for integers, non-empty for strings, actual value for booleans)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Bool(b) => *b,
        }
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Push(value) => write!(f, "PUSH {}", value),
            Instruction::Pop => write!(f, "POP"),
            Instruction::Dup => write!(f, "DUP"),
            Instruction::Add => write!(f, "ADD"),
            Instruction::Sub => write!(f, "SUB"),
            Instruction::Mul => write!(f, "MUL"),
            Instruction::Div => write!(f, "DIV"),
            Instruction::Equal => write!(f, "EQUAL"),
            Instruction::NotEqual => write!(f, "NOT_EQUAL"),
            Instruction::Less => write!(f, "LESS"),
            Instruction::Greater => write!(f, "GREATER"),
            Instruction::LessEqual => write!(f, "LESS_EQUAL"),
            Instruction::GreaterEqual => write!(f, "GREATER_EQUAL"),
            Instruction::And => write!(f, "AND"),
            Instruction::Or => write!(f, "OR"),
            Instruction::Not => write!(f, "NOT"),
            Instruction::LoadVar(n) => write!(f, "LOADVAR v${}", n),
            Instruction::StoreVar(n) => write!(f, "STOREVAR v${}", n),
            Instruction::LoadConst(n) => write!(f, "LOADCONST c${}", n),
            Instruction::Call(name, argc) => write!(f, "CALL {} {}", name, argc),
            Instruction::Return => write!(f, "RETURN"),
            Instruction::ReturnValue => write!(f, "RETURN_VALUE"),
            Instruction::Jump(addr) => write!(f, "JUMP {}", addr),
            Instruction::JumpIfFalse(addr) => write!(f, "JUMP_IF_FALSE {}", addr),
            Instruction::Print => write!(f, "PRINT"),
            Instruction::Describe(entity, desc) => write!(f, "DESCRIBE {} \"{}\"", entity, desc),
            Instruction::Halt => write!(f, "HALT"),
            Instruction::Nop => write!(f, "NOP"),
        }
    }
}

/// Bytecode program containing instructions and metadata
#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub entry_point: usize, // Instruction index where execution starts
}

impl Bytecode {
    pub fn new() -> Self {
        Bytecode {
            instructions: Vec::new(),
            constants: Vec::new(),
            entry_point: 0,
        }
    }

    /// Add an instruction and return its index
    pub fn emit(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    /// Add a constant value and return its index
    pub fn add_constant(&mut self, value: Value) -> u32 {
        let index = self.constants.len() as u32;
        self.constants.push(value);
        index
    }

    /// Set the entry point
    pub fn set_entry_point(&mut self, index: usize) {
        self.entry_point = index;
    }

    /// Get instruction at index
    pub fn get_instruction(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    /// Get constant at index
    pub fn get_constant(&self, index: u32) -> Option<&Value> {
        self.constants.get(index as usize)
    }

    /// Total number of instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if bytecode is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Disassemble bytecode for debugging
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("=== Bytecode Disassembly ===\n"));
        output.push_str(&format!("Entry point: {}\n", self.entry_point));
        output.push_str(&format!("Constants: {:?}\n\n", self.constants));

        for (i, instruction) in self.instructions.iter().enumerate() {
            let marker = if i == self.entry_point { ">" } else { " " };
            output.push_str(&format!("{} {:04} {}\n", marker, i, instruction));
        }

        output
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_display() {
        let inst = Instruction::Push(Value::Int(42));
        assert_eq!(inst.to_string(), "PUSH 42");

        let inst = Instruction::Push(Value::Bool(true));
        assert_eq!(inst.to_string(), "PUSH true");

        let inst = Instruction::LoadVar(5);
        assert_eq!(inst.to_string(), "LOADVAR v$5");

        let inst = Instruction::Call("f$0".to_string(), 2);
        assert_eq!(inst.to_string(), "CALL f$0 2");

        let inst = Instruction::Equal;
        assert_eq!(inst.to_string(), "EQUAL");

        let inst = Instruction::And;
        assert_eq!(inst.to_string(), "AND");

        let inst = Instruction::JumpIfFalse(42);
        assert_eq!(inst.to_string(), "JUMP_IF_FALSE 42");
    }

    #[test]
    fn test_value_operations() {
        let val = Value::Int(42);
        assert_eq!(val.as_int(), 42);
        assert!(val.is_truthy());
        assert_eq!(val.type_name(), "int");

        let zero = Value::Int(0);
        assert!(!zero.is_truthy());

        let bool_val = Value::Bool(true);
        assert!(bool_val.as_bool());
        assert!(bool_val.is_truthy());
        assert_eq!(bool_val.type_name(), "bool");

        let false_val = Value::Bool(false);
        assert!(!false_val.as_bool());
        assert!(!false_val.is_truthy());

        let str_val = Value::Str("hello".to_string());
        assert_eq!(str_val.as_str(), "hello");
        assert!(str_val.is_truthy());
        assert_eq!(str_val.type_name(), "str");

        let empty_str = Value::Str("".to_string());
        assert!(!empty_str.is_truthy());
    }

    #[test]
    fn test_value_conversions() {
        let int_val: Value = 42.into();
        assert_eq!(int_val, Value::Int(42));

        let bool_val: Value = true.into();
        assert_eq!(bool_val, Value::Bool(true));

        let str_val: Value = "hello".into();
        assert_eq!(str_val, Value::Str("hello".to_string()));

        let string_val: Value = "world".to_string().into();
        assert_eq!(string_val, Value::Str("world".to_string()));
    }

    #[test]
    fn test_bytecode_operations() {
        let mut bytecode = Bytecode::new();

        // Add some instructions
        let idx1 = bytecode.emit(Instruction::Push(Value::Int(42)));
        let idx2 = bytecode.emit(Instruction::Push(Value::Bool(true)));
        let idx3 = bytecode.emit(Instruction::And);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(bytecode.len(), 3);

        // Add constants
        let const_idx = bytecode.add_constant(Value::Str("test".to_string()));
        assert_eq!(const_idx, 0);
        assert_eq!(
            bytecode.get_constant(0),
            Some(&Value::Str("test".to_string()))
        );
    }

    #[test]
    fn test_disassembly() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Instruction::Push(Value::Bool(true)));
        bytecode.emit(Instruction::Push(Value::Bool(false)));
        bytecode.emit(Instruction::Or);
        bytecode.set_entry_point(0);

        let disasm = bytecode.disassemble();
        assert!(disasm.contains("PUSH true"));
        assert!(disasm.contains("PUSH false"));
        assert!(disasm.contains("OR"));
        assert!(disasm.contains("Entry point: 0"));
    }
}
