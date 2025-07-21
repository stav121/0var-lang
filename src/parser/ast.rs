//! Abstract Syntax Tree definitions for the zvar language

use crate::{span::Span, symbol_table::ValueType};

/// Top-level program structure
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level items (functions, main block, etc.)
#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    MainBlock(MainBlock),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Function(f) => f.span,
            Item::MainBlock(m) => m.span,
        }
    }
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String, // f$0, f$1, etc.
    pub params: Vec<Parameter>,
    pub return_type: ValueType,
    pub body: Block,
    pub span: Span,
    pub documentation: Option<String>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String, // v$0, v$1, etc.
    pub param_type: ValueType,
    pub span: Span,
}

/// Main block
#[derive(Debug, Clone)]
pub struct MainBlock {
    pub body: Block,
    pub span: Span,
    pub documentation: Option<String>,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Statements
#[derive(Debug, Clone)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    ConstantDeclaration(ConstantDeclaration),
    Assignment(Assignment),
    ExpressionStatement(Expression),
    Return(Return),
    Describe(Describe),
    If(IfStatement),
}

/// If statement: if (condition) { ... } else { ... }  -- NEW!
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

// Add Display implementations
impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::GreaterEqual => write!(f, ">="),
        }
    }
}

impl std::fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "&&"),
            LogicalOperator::Or => write!(f, "||"),
        }
    }
}

impl std::fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Not => write!(f, "!"),
        }
    }
}

// Helper constructors
impl LogicalExpression {
    pub fn new(left: Expression, operator: LogicalOperator, right: Expression, span: Span) -> Self {
        LogicalExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            span,
        }
    }
}

impl UnaryExpression {
    pub fn new(operator: UnaryOperator, operand: Expression, span: Span) -> Self {
        UnaryExpression {
            operator,
            operand: Box::new(operand),
            span,
        }
    }
}

impl IfStatement {
    pub fn new(
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    ) -> Self {
        IfStatement {
            condition,
            then_block,
            else_block,
            span,
        }
    }
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::VariableDeclaration(v) => v.span,
            Statement::ConstantDeclaration(c) => c.span,
            Statement::Assignment(a) => a.span,
            Statement::ExpressionStatement(e) => e.span(),
            Statement::Return(r) => r.span,
            Statement::Describe(d) => d.span,
            Statement::If(i) => i.span,
        }
    }
}

/// Variable declaration: int v$0 = 5;
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub value_type: ValueType,
    pub initializer: Option<Expression>,
    pub span: Span,
    pub documentation: Option<String>,
}

/// Constant declaration: int c$0 = 5;
#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: String,
    pub value_type: ValueType,
    pub initializer: Expression,
    pub span: Span,
    pub documentation: Option<String>,
}

/// Assignment: v$0 = 5;
#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: String,
    pub value: Expression,
    pub span: Span,
}

/// Return statement: ret v$0;
#[derive(Debug, Clone)]
pub struct Return {
    pub value: Option<Expression>,
    pub span: Span,
}

/// Describe statement: describe(v$0, "documentation");
#[derive(Debug, Clone)]
pub struct Describe {
    pub target: String,
    pub description: String,
    pub span: Span,
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expression {
    Integer(IntegerLiteral),
    String(StringLiteral),
    Boolean(BooleanLiteral),
    Variable(Variable),
    Binary(BinaryExpression),
    Logical(LogicalExpression),
    Unary(UnaryExpression),
    FunctionCall(FunctionCall),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Integer(i) => i.span,
            Expression::String(s) => s.span,
            Expression::Boolean(b) => b.span,
            Expression::Variable(v) => v.span,
            Expression::Binary(b) => b.span,
            Expression::Logical(l) => l.span,
            Expression::Unary(u) => u.span,
            Expression::FunctionCall(f) => f.span,
        }
    }
}

/// Integer literal: 42
#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub value: i64,
    pub span: Span,
}

/// String literal: "hello world"  -- NEW!
#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}

/// Boolean literal: true, false
#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    pub value: bool,
    pub span: Span,
}

/// Variable reference: v$0
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub span: Span,
}

/// Binary expression: v$0 + v$1
#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: Span,
}

/// Logical expression: v$0 && v$1
#[derive(Debug, Clone)]
pub struct LogicalExpression {
    pub left: Box<Expression>,
    pub operator: LogicalOperator,
    pub right: Box<Expression>,
    pub span: Span,
}

/// Unary expression: !v$0
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /

    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
}

/// Logical operators - NEW!
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And, // &&
    Or,  // ||
}

/// Unary operators - NEW!
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not, // !
}

/// Function call: f$0(v$1, v$2)
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

// Helper constructors for easier AST building
impl Program {
    pub fn new(items: Vec<Item>, span: Span) -> Self {
        Program { items, span }
    }
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<Parameter>,
        return_type: ValueType,
        body: Block,
        span: Span,
    ) -> Self {
        Function {
            name,
            params,
            return_type,
            body,
            span,
            documentation: None,
        }
    }

    pub fn with_documentation(mut self, doc: String) -> Self {
        self.documentation = Some(doc);
        self
    }
}

impl MainBlock {
    pub fn new(body: Block, span: Span) -> Self {
        MainBlock {
            body,
            span,
            documentation: None,
        }
    }

    pub fn with_documentation(mut self, doc: String) -> Self {
        self.documentation = Some(doc);
        self
    }
}

impl Block {
    pub fn new(statements: Vec<Statement>, span: Span) -> Self {
        Block { statements, span }
    }
}

impl BinaryExpression {
    pub fn new(left: Expression, operator: BinaryOperator, right: Expression, span: Span) -> Self {
        BinaryExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        let span = Span::new(1, 1, 1, 10);

        // Create a simple variable declaration
        let var_decl = VariableDeclaration {
            name: "v$0".to_string(),
            value_type: ValueType::Int,
            initializer: Some(Expression::Integer(IntegerLiteral { value: 42, span })),
            span,
            documentation: None,
        };

        assert_eq!(var_decl.name, "v$0");
        assert_eq!(var_decl.value_type, ValueType::Int);
    }

    #[test]
    fn test_string_literal() {
        let span = Span::new(1, 1, 1, 10);

        let string_lit = StringLiteral {
            value: "hello world".to_string(),
            span,
        };

        assert_eq!(string_lit.value, "hello world");
        assert_eq!(string_lit.span, span);
    }

    #[test]
    fn test_binary_expression() {
        let span = Span::new(1, 1, 1, 10);

        let left = Expression::Variable(Variable {
            name: "v$0".to_string(),
            span,
        });

        let right = Expression::Integer(IntegerLiteral { value: 5, span });

        let binary = BinaryExpression::new(left, BinaryOperator::Add, right, span);

        assert_eq!(binary.operator, BinaryOperator::Add);
        assert_eq!(binary.span, span);
    }

    #[test]
    fn test_function_construction() {
        let span = Span::new(1, 1, 5, 10);

        let param = Parameter {
            name: "v$0".to_string(),
            param_type: ValueType::Int,
            span,
        };

        let block = Block::new(vec![], span);

        let function = Function::new("f$0".to_string(), vec![param], ValueType::Int, block, span)
            .with_documentation("Test function".to_string());

        assert_eq!(function.name, "f$0");
        assert_eq!(function.params.len(), 1);
        assert_eq!(function.documentation, Some("Test function".to_string()));
    }
}
