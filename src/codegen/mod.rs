//! Code generation from AST to bytecode

pub mod debug_info;
pub mod instruction;

use crate::{
    error::{ZvarError, ZvarResult},
    parser::ast::*,
    symbol_table::SymbolTable,
};

use debug_info::DebugInfo;
use instruction::{Bytecode, Instruction, Value};
use std::collections::HashMap;

/// Code generator that converts AST to bytecode
pub struct CodeGenerator {
    bytecode: Bytecode,
    debug_info: DebugInfo,
    // Maps entity names to their runtime locations
    variable_slots: HashMap<String, u32>,
    constant_values: HashMap<String, Value>,
    next_variable_slot: u32,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            bytecode: Bytecode::new(),
            debug_info: DebugInfo::new(),
            variable_slots: HashMap::new(),
            constant_values: HashMap::new(),
            next_variable_slot: 0,
        }
    }

    /// Generate bytecode from a program
    pub fn generate(
        &mut self,
        program: &Program,
        symbol_table: &SymbolTable,
    ) -> ZvarResult<(Bytecode, DebugInfo)> {
        // First pass: collect all entities and assign slots
        self.collect_entities(program, symbol_table)?;

        // Second pass: generate code
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.generate_function(func)?;
                }
                Item::MainBlock(main) => {
                    // Main block is the entry point
                    let start_index = self.bytecode.len();
                    self.bytecode.set_entry_point(start_index);
                    self.debug_info
                        .mark_function_start("main".to_string(), start_index);

                    self.generate_block(&main.body)?;

                    // End main with halt
                    self.emit_with_span(Instruction::Halt, main.span);
                }
            }
        }

        Ok((self.bytecode.clone(), self.debug_info.clone()))
    }

    /// First pass: collect all entities and assign runtime slots
    fn collect_entities(
        &mut self,
        program: &Program,
        symbol_table: &SymbolTable,
    ) -> ZvarResult<()> {
        // Collect from symbol table
        for (name, symbol) in symbol_table.all_symbols() {
            match &symbol.entity_type {
                crate::symbol_table::EntityType::Variable { .. } => {
                    // Assign a runtime slot for variables
                    if name.starts_with("v$") {
                        let slot = self.next_variable_slot;
                        self.variable_slots.insert(name.clone(), slot);
                        self.next_variable_slot += 1;
                    }
                }
                crate::symbol_table::EntityType::Constant { .. } => {
                    // Constants need slots too for now (we could optimize this later)
                    if name.starts_with("c$") {
                        let slot = self.next_variable_slot;
                        self.variable_slots.insert(name.clone(), slot);
                        self.next_variable_slot += 1;
                    }
                }
                crate::symbol_table::EntityType::Function { .. } => {
                    // Functions are handled separately
                }
            }

            // Store documentation
            if let Some(doc) = &symbol.documentation {
                self.debug_info.add_entity_doc(name.clone(), doc.clone());
            }
        }

        // Also collect from AST to catch any missed variables
        self.collect_from_ast(program)?;

        Ok(())
    }

    /// Additional collection from AST nodes
    fn collect_from_ast(&mut self, program: &Program) -> ZvarResult<()> {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.collect_from_block(&func.body)?;
                    // Also collect function parameters
                    for param in &func.params {
                        if !self.variable_slots.contains_key(&param.name) {
                            let slot = self.next_variable_slot;
                            self.variable_slots.insert(param.name.clone(), slot);
                            self.next_variable_slot += 1;
                        }
                    }
                }
                Item::MainBlock(main) => {
                    self.collect_from_block(&main.body)?;
                }
            }
        }
        Ok(())
    }

    /// Collect variables from a block
    fn collect_from_block(&mut self, block: &Block) -> ZvarResult<()> {
        for stmt in &block.statements {
            self.collect_from_statement(stmt)?;
        }
        Ok(())
    }

    /// Collect variables from a statement
    fn collect_from_statement(&mut self, stmt: &Statement) -> ZvarResult<()> {
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                if !self.variable_slots.contains_key(&var_decl.name) {
                    let slot = self.next_variable_slot;
                    self.variable_slots.insert(var_decl.name.clone(), slot);
                    self.next_variable_slot += 1;
                }
                if let Some(init) = &var_decl.initializer {
                    self.collect_from_expression(init)?;
                }
            }
            Statement::ConstantDeclaration(const_decl) => {
                if !self.variable_slots.contains_key(&const_decl.name) {
                    let slot = self.next_variable_slot;
                    self.variable_slots.insert(const_decl.name.clone(), slot);
                    self.next_variable_slot += 1;
                }
                self.collect_from_expression(&const_decl.initializer)?;
            }
            Statement::Assignment(assignment) => {
                if !self.variable_slots.contains_key(&assignment.target) {
                    let slot = self.next_variable_slot;
                    self.variable_slots.insert(assignment.target.clone(), slot);
                    self.next_variable_slot += 1;
                }
                self.collect_from_expression(&assignment.value)?;
            }
            Statement::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.collect_from_expression(value)?;
                }
            }
            Statement::ExpressionStatement(expr) => {
                self.collect_from_expression(expr)?;
            }
            Statement::Describe(_) => {
                // Nothing to collect from describe statements
            }
            Statement::If(if_stmt) => {
                self.collect_from_expression(&if_stmt.condition)?;
                self.collect_from_block(&if_stmt.then_block)?;
                if let Some(else_block) = &if_stmt.else_block {
                    self.collect_from_block(else_block)?;
                }
            }
        }
        Ok(())
    }

    /// Collect variables from an expression
    fn collect_from_expression(&mut self, expr: &Expression) -> ZvarResult<()> {
        match expr {
            Expression::Variable(var) => {
                if !self.variable_slots.contains_key(&var.name) {
                    let slot = self.next_variable_slot;
                    self.variable_slots.insert(var.name.clone(), slot);
                    self.next_variable_slot += 1;
                }
            }
            Expression::Binary(binary) => {
                self.collect_from_expression(&binary.left)?;
                self.collect_from_expression(&binary.right)?;
            }
            Expression::Logical(logical) => {
                self.collect_from_expression(&logical.left)?;
                self.collect_from_expression(&logical.right)?;
            }
            Expression::Unary(unary) => {
                self.collect_from_expression(&unary.operand)?;
            }
            Expression::FunctionCall(call) => {
                for arg in &call.arguments {
                    self.collect_from_expression(arg)?;
                }
            }
            Expression::Integer(_) => {
                // Nothing to collect from integer literals
            }
            Expression::String(_) => {
                // Nothing to collect from string literals
            }
            Expression::Boolean(_) => {
                // Nothing to collect from boolean literals
            }
        }
        Ok(())
    }

    /// Generate code for a function
    fn generate_function(&mut self, func: &Function) -> ZvarResult<()> {
        let start_index = self.bytecode.len();
        self.debug_info
            .mark_function_start(func.name.clone(), start_index);

        // Generate function body
        self.generate_block(&func.body)?;

        // If no explicit return, add implicit return
        if !matches!(
            self.bytecode.instructions.last(),
            Some(Instruction::Return | Instruction::ReturnValue)
        ) {
            self.emit_with_span(Instruction::Return, func.span);
        }

        Ok(())
    }

    /// Generate code for a block
    fn generate_block(&mut self, block: &Block) -> ZvarResult<()> {
        for statement in &block.statements {
            self.generate_statement(statement)?;
        }
        Ok(())
    }

    /// Generate code for a statement
    fn generate_statement(&mut self, stmt: &Statement) -> ZvarResult<()> {
        match stmt {
            Statement::If(if_stmt) => {
                // Generate condition
                self.generate_expression(&if_stmt.condition)?;

                // Jump to else block if condition is false
                let else_jump = self.bytecode.len();
                self.emit_with_span(Instruction::JumpIfFalse(0), if_stmt.span); // Placeholder address

                // Generate then block
                self.generate_block(&if_stmt.then_block)?;

                if let Some(else_block) = &if_stmt.else_block {
                    // Jump over else block after then block
                    let end_jump = self.bytecode.len();
                    self.emit_with_span(Instruction::Jump(0), if_stmt.span); // Placeholder address

                    // Update else jump to point here
                    let else_target = self.bytecode.len();
                    if let Some(Instruction::JumpIfFalse(ref mut addr)) =
                        self.bytecode.instructions.get_mut(else_jump)
                    {
                        *addr = else_target;
                    }

                    // Generate else block
                    self.generate_block(else_block)?;

                    // Update end jump to point here
                    let end_target = self.bytecode.len();
                    if let Some(Instruction::Jump(ref mut addr)) =
                        self.bytecode.instructions.get_mut(end_jump)
                    {
                        *addr = end_target;
                    }
                } else {
                    // No else block, just update the jump to point to end
                    let end_target = self.bytecode.len();
                    if let Some(Instruction::JumpIfFalse(ref mut addr)) =
                        self.bytecode.instructions.get_mut(else_jump)
                    {
                        *addr = end_target;
                    }
                }
            }

            Statement::VariableDeclaration(var_decl) => {
                if let Some(init) = &var_decl.initializer {
                    // Generate initializer expression
                    self.generate_expression(init)?;

                    // Store in variable slot
                    if let Some(&slot) = self.variable_slots.get(&var_decl.name) {
                        self.emit_with_span(Instruction::StoreVar(slot), var_decl.span);
                    } else {
                        return Err(ZvarError::CodegenError {
                            message: format!("Variable {} not found in slots", var_decl.name),
                        });
                    }
                }
            }

            Statement::ConstantDeclaration(const_decl) => {
                // Generate initializer expression
                self.generate_expression(&const_decl.initializer)?;

                // Store in variable slot (constants use same mechanism as variables)
                if let Some(&slot) = self.variable_slots.get(&const_decl.name) {
                    self.emit_with_span(Instruction::StoreVar(slot), const_decl.span);
                } else {
                    return Err(ZvarError::CodegenError {
                        message: format!("Constant {} not found in slots", const_decl.name),
                    });
                }
            }

            Statement::Assignment(assignment) => {
                // Generate value expression
                self.generate_expression(&assignment.value)?;

                // Store in variable slot
                if let Some(&slot) = self.variable_slots.get(&assignment.target) {
                    self.emit_with_span(Instruction::StoreVar(slot), assignment.span);
                } else {
                    return Err(ZvarError::CodegenError {
                        message: format!("Variable {} not found in slots", assignment.target),
                    });
                }
            }

            Statement::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.generate_expression(value)?;
                    self.emit_with_span(Instruction::ReturnValue, ret.span);
                } else {
                    self.emit_with_span(Instruction::Return, ret.span);
                }
            }

            Statement::Describe(desc) => {
                // Generate describe instruction for runtime
                let instruction =
                    Instruction::Describe(desc.target.clone(), desc.description.clone());
                self.emit_with_span(instruction, desc.span);
            }

            Statement::ExpressionStatement(expr) => {
                self.generate_expression(expr)?;
                // Only pop the result if it's not a function call that consumes its arguments
                // For example, print() already consumes its argument, so we don't need to pop
                match expr {
                    Expression::FunctionCall(call) => {
                        // Built-in functions like print() handle their own stack management
                        if call.name == "print" {
                            // print() consumes its argument, no need to pop
                        } else {
                            // User-defined functions might leave a return value on the stack
                            // For now, we'll pop it since expression statements don't use the result
                            self.emit_with_span(Instruction::Pop, expr.span());
                        }
                    }
                    _ => {
                        // Other expressions leave their result on the stack, so we need to pop it
                        self.emit_with_span(Instruction::Pop, expr.span());
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate code for an expression
    fn generate_expression(&mut self, expr: &Expression) -> ZvarResult<()> {
        match expr {
            Expression::Integer(int_lit) => {
                let value = Value::Int(int_lit.value);
                self.emit_with_span(Instruction::Push(value), int_lit.span);
            }

            Expression::String(str_lit) => {
                let value = Value::Str(str_lit.value.clone());
                self.emit_with_span(Instruction::Push(value), str_lit.span);
            }

            Expression::Boolean(bool_lit) => {
                // NEW!
                let value = Value::Bool(bool_lit.value);
                self.emit_with_span(Instruction::Push(value), bool_lit.span);
            }

            Expression::Variable(var) => {
                if let Some(&slot) = self.variable_slots.get(&var.name) {
                    self.emit_with_span(Instruction::LoadVar(slot), var.span);
                } else {
                    return Err(ZvarError::CodegenError {
                        message: format!("Variable {} not found in slots", var.name),
                    });
                }
            }

            Expression::Binary(binary) => {
                // Generate left operand
                self.generate_expression(&binary.left)?;

                // Generate right operand
                self.generate_expression(&binary.right)?;

                // Generate operator instruction
                let instruction = match binary.operator {
                    BinaryOperator::Add => Instruction::Add,
                    BinaryOperator::Subtract => Instruction::Sub,
                    BinaryOperator::Multiply => Instruction::Mul,
                    BinaryOperator::Divide => Instruction::Div,
                    BinaryOperator::Equal => Instruction::Equal, // NEW!
                    BinaryOperator::NotEqual => Instruction::NotEqual, // NEW!
                    BinaryOperator::Less => Instruction::Less,   // NEW!
                    BinaryOperator::Greater => Instruction::Greater, // NEW!
                    BinaryOperator::LessEqual => Instruction::LessEqual, // NEW!
                    BinaryOperator::GreaterEqual => Instruction::GreaterEqual, // NEW!
                };

                self.emit_with_span(instruction, binary.span);
            }

            Expression::Logical(logical) => {
                // NEW!
                // Generate left operand
                self.generate_expression(&logical.left)?;

                // Generate right operand
                self.generate_expression(&logical.right)?;

                // Generate logical operator instruction
                let instruction = match logical.operator {
                    LogicalOperator::And => Instruction::And,
                    LogicalOperator::Or => Instruction::Or,
                };

                self.emit_with_span(instruction, logical.span);
            }

            Expression::Unary(unary) => {
                // NEW!
                // Generate operand
                self.generate_expression(&unary.operand)?;

                // Generate unary operator instruction
                let instruction = match unary.operator {
                    UnaryOperator::Not => Instruction::Not,
                };

                self.emit_with_span(instruction, unary.span);
            }

            Expression::FunctionCall(call) => {
                // Generate arguments in order
                for arg in &call.arguments {
                    self.generate_expression(arg)?;
                }

                // Generate call instruction
                let argc = call.arguments.len() as u32;

                if call.name == "print" {
                    // Special handling for built-in print function
                    if argc != 1 {
                        return Err(ZvarError::WrongArgumentCount {
                            span: call.span,
                            name: call.name.clone(),
                            expected: 1,
                            found: argc as usize,
                        });
                    }
                    self.emit_with_span(Instruction::Print, call.span);
                } else {
                    // Regular function call
                    self.emit_with_span(Instruction::Call(call.name.clone(), argc), call.span);
                }
            }
        }

        Ok(())
    }

    /// Emit an instruction with debug span information
    fn emit_with_span(&mut self, instruction: Instruction, span: crate::span::Span) -> usize {
        let index = self.bytecode.emit(instruction);
        self.debug_info.add_instruction_span(index, span);
        index
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::ast::*, span::Span, symbol_table::SymbolTable};

    #[test]
    fn test_variable_slot_assignment() {
        let mut codegen = CodeGenerator::new();

        // Manually add some variables to test slot assignment
        codegen.variable_slots.insert("v$0".to_string(), 0);
        codegen.variable_slots.insert("v$1".to_string(), 1);
        codegen.next_variable_slot = 2;

        assert_eq!(codegen.variable_slots.get("v$0"), Some(&0));
        assert_eq!(codegen.variable_slots.get("v$1"), Some(&1));
    }

    #[test]
    fn test_simple_expression_generation() {
        let mut codegen = CodeGenerator::new();

        // Create simple integer expression: 42
        let expr = Expression::Integer(IntegerLiteral {
            value: 42,
            span: Span::new(1, 1, 1, 2),
        });

        codegen.generate_expression(&expr).unwrap();

        assert_eq!(codegen.bytecode.instructions.len(), 1);
        match &codegen.bytecode.instructions[0] {
            Instruction::Push(Value::Int(42)) => (),
            _ => panic!("Expected PUSH 42"),
        }
    }

    #[test]
    fn test_binary_expression_generation() {
        let mut codegen = CodeGenerator::new();

        // Create binary expression: 1 + 2
        let left = Expression::Integer(IntegerLiteral {
            value: 1,
            span: Span::new(1, 1, 1, 1),
        });
        let right = Expression::Integer(IntegerLiteral {
            value: 2,
            span: Span::new(1, 5, 1, 5),
        });

        let binary = Expression::Binary(BinaryExpression::new(
            left,
            BinaryOperator::Add,
            right,
            Span::new(1, 1, 1, 5),
        ));

        codegen.generate_expression(&binary).unwrap();

        assert_eq!(codegen.bytecode.instructions.len(), 3);
        assert!(matches!(
            codegen.bytecode.instructions[0],
            Instruction::Push(Value::Int(1))
        ));
        assert!(matches!(
            codegen.bytecode.instructions[1],
            Instruction::Push(Value::Int(2))
        ));
        assert!(matches!(codegen.bytecode.instructions[2], Instruction::Add));
    }
}
