//! Parser for the zvar language
//!
//! Converts a stream of tokens into an Abstract Syntax Tree (AST)

pub mod ast;

use crate::{
    error::{ZvarError, ZvarResult},
    lexer::{token::Token, Lexer},
    span::Span,
    symbol_table::{EntityType, Symbol, SymbolTable, ValueType},
};

use ast::*;

/// Recursive descent parser for zvar
pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    symbol_table: &'a mut SymbolTable,
}

impl<'a> Parser<'a> {
    /// Create a new parser from source code
    pub fn new(source: &str, symbol_table: &'a mut SymbolTable) -> ZvarResult<Self> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        Ok(Parser {
            tokens,
            current: 0,
            symbol_table,
        })
    }

    /// Get the current token without advancing
    fn current_token(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::Eof)
    }

    /// Get the previous token
    fn previous_token(&self) -> &Token {
        if self.current > 0 {
            &self.tokens[self.current - 1]
        } else {
            &Token::Eof
        }
    }

    /// Check if we're at the end
    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Token::Eof)
    }

    /// Advance to the next token and return the previous one
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous_token()
    }

    /// Check if current token matches any of the given tokens
    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(token_type)
    }

    /// Consume a token if it matches, otherwise return error
    fn consume(&mut self, expected: Token, _message: &str) -> ZvarResult<&Token> {
        if self.check(&expected) {
            Ok(self.advance())
        } else {
            Err(ZvarError::UnexpectedToken {
                span: self.current_span(),
                expected: expected.to_string(),
                found: self.current_token().to_string(),
            })
        }
    }

    /// Get a span for the current token
    fn current_span(&self) -> Span {
        // For now, we'll use a dummy span. In a real implementation,
        // we'd need to track spans through the lexer
        Span::new(1, 1, 1, 1)
    }

    /// Skip newlines and comments
    fn skip_newlines(&mut self) {
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }
    }

    /// Collect documentation comments
    fn collect_documentation(&mut self) -> Option<String> {
        let mut docs = Vec::new();

        while let Token::DocComment(comment) = self.current_token() {
            docs.push(comment.clone());
            self.advance();
            self.skip_newlines();
        }

        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }

    /// Parse the entire program
    pub fn parse_program(&mut self) -> ZvarResult<Program> {
        let start_span = self.current_span();
        let mut items = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            // Collect any documentation comments
            if let Some(doc) = self.collect_documentation() {
                self.symbol_table.add_pending_doc(doc);
            }

            let item = self.parse_item()?;
            items.push(item);

            self.skip_newlines();
        }

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        Ok(Program::new(items, span))
    }

    /// Parse a top-level item (function or main block)
    fn parse_item(&mut self) -> ZvarResult<Item> {
        match self.current_token() {
            Token::Fn => {
                let function = self.parse_function()?;
                Ok(Item::Function(function))
            }
            Token::Main => {
                let main_block = self.parse_main_block()?;
                Ok(Item::MainBlock(main_block))
            }
            _ => Err(ZvarError::UnexpectedToken {
                span: self.current_span(),
                expected: "fn or main".to_string(),
                found: self.current_token().to_string(),
            }),
        }
    }

    /// Parse a function definition
    fn parse_function(&mut self) -> ZvarResult<Function> {
        let start_span = self.current_span();

        // fn
        self.consume(Token::Fn, "Expected 'fn'")?;

        // Function name (f$N)
        let name = match self.current_token() {
            Token::Function(n) => {
                let name = format!("f${}", n);
                self.advance();
                name
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "function name (f$N)".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        // Parameters
        self.consume(Token::LeftParen, "Expected '('")?;
        let mut params = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let param = self.parse_parameter()?;
                params.push(param);

                if self.check(&Token::Comma) {
                    self.advance(); // consume comma
                } else {
                    break;
                }
            }
        }

        self.consume(Token::RightParen, "Expected ')'")?;

        // Return type
        self.consume(Token::Arrow, "Expected '->'")?;
        let return_type = self.parse_type()?;

        // ADD FUNCTION TO SYMBOL TABLE BEFORE PARSING BODY
        let func_symbol = Symbol::new(
            EntityType::Function {
                params: params.iter().map(|p| p.param_type.clone()).collect(),
                return_type: return_type.clone(),
            },
            start_span,
        );
        self.symbol_table.define(name.clone(), func_symbol)?;

        // Enter function scope
        self.symbol_table.enter_scope();

        // Add parameters to symbol table
        for param in &params {
            let symbol = Symbol::new(
                EntityType::Variable {
                    value_type: param.param_type.clone(),
                },
                param.span,
            );
            self.symbol_table.define(param.name.clone(), symbol)?;
        }

        // Parse function body
        let body = self.parse_block()?;

        // Exit function scope
        self.symbol_table.exit_scope();

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        let mut function = Function::new(name, params, return_type, body, span);

        // Attach documentation if any
        if let Some(docs) = self.symbol_table.take_pending_docs() {
            function = function.with_documentation(docs);
        }

        Ok(function)
    }

    /// Parse a function parameter
    fn parse_parameter(&mut self) -> ZvarResult<Parameter> {
        let start_span = self.current_span();

        // Parameter name (v$N)
        let name = match self.current_token() {
            Token::Variable(n) => {
                let name = format!("v${}", n);
                self.advance();
                name
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "parameter name (v$N)".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        // Parameter type
        let param_type = self.parse_type()?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        Ok(Parameter {
            name,
            param_type,
            span,
        })
    }

    /// Parse a main block
    fn parse_main_block(&mut self) -> ZvarResult<MainBlock> {
        let start_span = self.current_span();

        // main
        self.consume(Token::Main, "Expected 'main'")?;

        // Enter main scope
        self.symbol_table.enter_scope();

        // Parse body
        let body = self.parse_block()?;

        // Exit main scope
        self.symbol_table.exit_scope();

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        let mut main_block = MainBlock::new(body, span);

        // Attach documentation if any
        if let Some(docs) = self.symbol_table.take_pending_docs() {
            main_block = main_block.with_documentation(docs);
        }

        Ok(main_block)
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> ZvarResult<Block> {
        let start_span = self.current_span();

        self.consume(Token::LeftBrace, "Expected '{'")?;
        self.skip_newlines();

        let mut statements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Collect documentation for next statement
            if let Some(doc) = self.collect_documentation() {
                self.symbol_table.add_pending_doc(doc);
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);

            self.skip_newlines();
        }

        self.consume(Token::RightBrace, "Expected '}'")?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        Ok(Block::new(statements, span))
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> ZvarResult<Statement> {
        match self.current_token() {
            Token::Int | Token::Str | Token::Bool => {
                // Could be variable or constant declaration
                let value_type = match self.current_token() {
                    Token::Int => {
                        self.advance();
                        ValueType::Int
                    }
                    Token::Str => {
                        self.advance();
                        ValueType::Str
                    }
                    Token::Bool => {
                        self.advance();
                        ValueType::Bool
                    }
                    _ => unreachable!(),
                };

                match self.current_token() {
                    Token::Variable(_) => {
                        let var_decl = self.parse_variable_declaration_after_type(value_type)?;
                        Ok(Statement::VariableDeclaration(var_decl))
                    }
                    Token::Constant(_) => {
                        let const_decl = self.parse_constant_declaration_after_type(value_type)?;
                        Ok(Statement::ConstantDeclaration(const_decl))
                    }
                    _ => Err(ZvarError::UnexpectedToken {
                        span: self.current_span(),
                        expected: "variable or constant name".to_string(),
                        found: self.current_token().to_string(),
                    }),
                }
            }
            Token::Variable(_) => {
                // Assignment
                let assignment = self.parse_assignment()?;
                Ok(Statement::Assignment(assignment))
            }
            Token::Ret => {
                let return_stmt = self.parse_return()?;
                Ok(Statement::Return(return_stmt))
            }
            Token::Describe => {
                let describe_stmt = self.parse_describe()?;
                Ok(Statement::Describe(describe_stmt))
            }
            Token::If => {
                let if_stmt = self.parse_if_statement()?;
                Ok(Statement::If(if_stmt))
            }
            _ => {
                // Expression statement
                let expr = self.parse_expression()?;
                self.consume(Token::Semicolon, "Expected ';'")?;
                Ok(Statement::ExpressionStatement(expr))
            }
        }
    }

    /// Parse if statement
    fn parse_if_statement(&mut self) -> ZvarResult<IfStatement> {
        let start_span = self.current_span();

        self.consume(Token::If, "Expected 'if'")?;
        self.consume(Token::LeftParen, "Expected '('")?;

        let condition = self.parse_expression()?;

        self.consume(Token::RightParen, "Expected ')'")?;

        let then_block = self.parse_block()?;

        let else_block = if self.check(&Token::Else) {
            self.advance(); // consume 'else'
            Some(self.parse_block()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        Ok(IfStatement::new(condition, then_block, else_block, span))
    }

    /// Parse variable declaration after type has been consumed
    fn parse_variable_declaration_after_type(
        &mut self,
        value_type: ValueType,
    ) -> ZvarResult<VariableDeclaration> {
        let start_span = self.current_span();

        // Variable name
        let name = match self.current_token() {
            Token::Variable(n) => {
                let name = format!("v${}", n);
                self.advance();
                name
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "variable name (v$N)".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        // Optional initializer
        let initializer = if self.check(&Token::Assign) {
            self.advance(); // consume '='
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(Token::Semicolon, "Expected ';'")?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        // Add to symbol table
        let mut symbol = Symbol::new(
            EntityType::Variable {
                value_type: value_type.clone(),
            },
            span,
        );

        if initializer.is_some() {
            symbol = symbol.mark_initialized();
        }

        self.symbol_table.define(name.clone(), symbol)?;

        let mut var_decl = VariableDeclaration {
            name,
            value_type,
            initializer,
            span,
            documentation: None,
        };

        // Attach documentation if any
        if let Some(docs) = self.symbol_table.take_pending_docs() {
            var_decl.documentation = Some(docs);
        }

        Ok(var_decl)
    }

    /// Parse constant declaration after type has been consumed
    fn parse_constant_declaration_after_type(
        &mut self,
        value_type: ValueType,
    ) -> ZvarResult<ConstantDeclaration> {
        let start_span = self.current_span();

        // Constant name
        let name = match self.current_token() {
            Token::Constant(n) => {
                let name = format!("c${}", n);
                self.advance();
                name
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "constant name (c$N)".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        // Constants must have initializer
        self.consume(
            Token::Assign,
            "Expected '=' (constants must be initialized)",
        )?;
        let initializer = self.parse_expression()?;

        self.consume(Token::Semicolon, "Expected ';'")?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        // Add to symbol table
        let symbol = Symbol::new(
            EntityType::Constant {
                value_type: value_type.clone(),
            },
            span,
        )
        .mark_initialized();

        self.symbol_table.define(name.clone(), symbol)?;

        let mut const_decl = ConstantDeclaration {
            name,
            value_type,
            initializer,
            span,
            documentation: None,
        };

        // Attach documentation if any
        if let Some(docs) = self.symbol_table.take_pending_docs() {
            const_decl.documentation = Some(docs);
        }

        Ok(const_decl)
    }

    /// Parse assignment statement
    fn parse_assignment(&mut self) -> ZvarResult<Assignment> {
        let start_span = self.current_span();

        // Target variable
        let target = match self.current_token() {
            Token::Variable(n) => {
                let name = format!("v${}", n);
                self.advance();
                name
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "variable name (v$N)".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        // Check if target exists and is not a constant
        if let Some(symbol) = self.symbol_table.lookup(&target) {
            if symbol.is_constant() {
                return Err(ZvarError::CannotAssignToConstant {
                    span: self.current_span(),
                    name: target,
                });
            }
        } else {
            return Err(ZvarError::UndefinedEntity {
                span: self.current_span(),
                name: target,
            });
        }

        self.consume(Token::Assign, "Expected '='")?;
        let value = self.parse_expression()?;
        self.consume(Token::Semicolon, "Expected ';'")?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        Ok(Assignment {
            target,
            value,
            span,
        })
    }

    /// Parse return statement
    fn parse_return(&mut self) -> ZvarResult<Return> {
        let start_span = self.current_span();

        self.consume(Token::Ret, "Expected 'ret'")?;

        let value = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(Token::Semicolon, "Expected ';'")?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        Ok(Return { value, span })
    }

    /// Parse describe statement
    fn parse_describe(&mut self) -> ZvarResult<Describe> {
        let start_span = self.current_span();

        self.consume(Token::Describe, "Expected 'describe'")?;
        self.consume(Token::LeftParen, "Expected '('")?;

        // Target entity (don't validate existence yet)
        let target = match self.current_token() {
            Token::Variable(n) => {
                let name = format!("v${}", n);
                self.advance();
                name
            }
            Token::Constant(n) => {
                let name = format!("c${}", n);
                self.advance();
                name
            }
            Token::Function(n) => {
                let name = format!("f${}", n);
                self.advance();
                name
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "entity name (v$N, c$N, or f$N)".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        self.consume(Token::Comma, "Expected ','")?;

        // Description string
        let description = match self.current_token() {
            Token::String(s) => {
                let desc = s.clone();
                self.advance();
                desc
            }
            _ => {
                return Err(ZvarError::UnexpectedToken {
                    span: self.current_span(),
                    expected: "string literal".to_string(),
                    found: self.current_token().to_string(),
                });
            }
        };

        self.consume(Token::RightParen, "Expected ')'")?;
        self.consume(Token::Semicolon, "Expected ';'")?;

        let end_span = self.current_span();
        let span = Span::from_to(start_span, end_span);

        // Try to add documentation, but don't fail if entity doesn't exist yet
        let _ = self
            .symbol_table
            .add_documentation(&target, description.clone());

        Ok(Describe {
            target,
            description,
            span,
        })
    }

    /// Parse a type
    fn parse_type(&mut self) -> ZvarResult<ValueType> {
        match self.current_token() {
            Token::Int => {
                self.advance();
                Ok(ValueType::Int)
            }
            Token::Str => {
                self.advance();
                Ok(ValueType::Str)
            }
            Token::Bool => {
                self.advance();
                Ok(ValueType::Bool)
            }
            _ => Err(ZvarError::UnexpectedToken {
                span: self.current_span(),
                expected: "type".to_string(),
                found: self.current_token().to_string(),
            }),
        }
    }

    /// Parse an expression (updated with precedence for logical operators)
    fn parse_expression(&mut self) -> ZvarResult<Expression> {
        self.parse_logical_or()
    }

    /// Parse logical OR expressions
    fn parse_logical_or(&mut self) -> ZvarResult<Expression> {
        let mut expr = self.parse_logical_and()?;

        while matches!(self.current_token(), Token::Or) {
            let operator = LogicalOperator::Or;
            self.advance();
            let right = self.parse_logical_and()?;
            let span = Span::from_to(expr.span(), right.span());

            expr = Expression::Logical(LogicalExpression::new(expr, operator, right, span));
        }

        Ok(expr)
    }

    /// Parse logical AND expressions
    fn parse_logical_and(&mut self) -> ZvarResult<Expression> {
        let mut expr = self.parse_equality()?;

        while matches!(self.current_token(), Token::And) {
            let operator = LogicalOperator::And;
            self.advance();
            let right = self.parse_equality()?;
            let span = Span::from_to(expr.span(), right.span());

            expr = Expression::Logical(LogicalExpression::new(expr, operator, right, span));
        }

        Ok(expr)
    }

    /// Parse equality expressions
    fn parse_equality(&mut self) -> ZvarResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while matches!(self.current_token(), Token::Equal | Token::NotEqual) {
            let operator = match self.current_token() {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_comparison()?;
            let span = Span::from_to(expr.span(), right.span());

            expr = Expression::Binary(BinaryExpression::new(expr, operator, right, span));
        }

        Ok(expr)
    }

    /// Parse comparison expressions
    fn parse_comparison(&mut self) -> ZvarResult<Expression> {
        let mut expr = self.parse_additive()?;

        while matches!(
            self.current_token(),
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual
        ) {
            let operator = match self.current_token() {
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_additive()?;
            let span = Span::from_to(expr.span(), right.span());

            expr = Expression::Binary(BinaryExpression::new(expr, operator, right, span));
        }

        Ok(expr)
    }

    /// Parse additive expressions (+ and -)
    fn parse_additive(&mut self) -> ZvarResult<Expression> {
        let mut expr = self.parse_multiplicative()?;

        while matches!(self.current_token(), Token::Plus | Token::Minus) {
            let operator = match self.current_token() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_multiplicative()?;
            let span = Span::from_to(expr.span(), right.span());

            expr = Expression::Binary(BinaryExpression::new(expr, operator, right, span));
        }

        Ok(expr)
    }

    /// Parse multiplicative expressions (* and /)
    fn parse_multiplicative(&mut self) -> ZvarResult<Expression> {
        let mut expr = self.parse_unary()?;

        while matches!(self.current_token(), Token::Multiply | Token::Divide) {
            let operator = match self.current_token() {
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_unary()?;
            let span = Span::from_to(expr.span(), right.span());

            expr = Expression::Binary(BinaryExpression::new(expr, operator, right, span));
        }

        Ok(expr)
    }

    /// Parse unary expressions
    fn parse_unary(&mut self) -> ZvarResult<Expression> {
        match self.current_token() {
            Token::Not => {
                let operator = UnaryOperator::Not;
                let start_span = self.current_span();
                self.advance();
                let operand = self.parse_unary()?;
                let span = Span::from_to(start_span, operand.span());
                Ok(Expression::Unary(UnaryExpression::new(
                    operator, operand, span,
                )))
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse primary expressions
    fn parse_primary(&mut self) -> ZvarResult<Expression> {
        let span = self.current_span();

        match self.current_token() {
            Token::Integer(value) => {
                let value = *value;
                self.advance();
                Ok(Expression::Integer(IntegerLiteral { value, span }))
            }
            Token::String(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expression::String(StringLiteral { value, span }))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Boolean(BooleanLiteral { value: true, span }))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Boolean(BooleanLiteral { value: false, span }))
            }
            Token::Variable(n) => {
                let name = format!("v${}", n);
                self.advance();

                // Check if it's actually a function call
                if self.check(&Token::LeftParen) {
                    return Err(ZvarError::UnexpectedToken {
                        span,
                        expected: "function name (f$N) for function call".to_string(),
                        found: name,
                    });
                }

                Ok(Expression::Variable(Variable { name, span }))
            }
            Token::Constant(n) => {
                let name = format!("c${}", n);
                self.advance();
                Ok(Expression::Variable(Variable { name, span }))
            }
            Token::Function(n) => {
                let name = format!("f${}", n);
                self.advance();

                // Must be a function call
                self.consume(Token::LeftParen, "Expected '(' after function name")?;

                let mut arguments = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        let arg = self.parse_expression()?;
                        arguments.push(arg);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }

                self.consume(Token::RightParen, "Expected ')'")?;

                let end_span = self.current_span();
                let call_span = Span::from_to(span, end_span);

                Ok(Expression::FunctionCall(FunctionCall {
                    name,
                    arguments,
                    span: call_span,
                }))
            }
            Token::Print => {
                let name = "print".to_string();
                self.advance();

                self.consume(Token::LeftParen, "Expected '(' after 'print'")?;

                let mut arguments = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        let arg = self.parse_expression()?;
                        arguments.push(arg);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }

                self.consume(Token::RightParen, "Expected ')'")?;
                let end_span = self.current_span();
                let call_span = Span::from_to(span, end_span);

                Ok(Expression::FunctionCall(FunctionCall {
                    name,
                    arguments,
                    span: call_span,
                }))
            }
            Token::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')'")?;
                Ok(expr)
            }
            _ => Err(ZvarError::UnexpectedToken {
                span,
                expected: "expression".to_string(),
                found: self.current_token().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_main() {
        let source = r#"
        main {
            int v$0 = 1;
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::MainBlock(main) => {
                assert_eq!(main.body.statements.len(), 1);
            }
            _ => panic!("Expected main block"),
        }
    }

    #[test]
    fn test_parse_function() {
        let source = r#"
        fn f$0(v$0 int, v$1 int) -> int {
            ret v$0 + v$1;
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name, "f$0");
                assert_eq!(func.params.len(), 2);
                assert_eq!(func.return_type, ValueType::Int);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = r#"
        main {
            int v$0 = 1 + 2 * 3;
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        // Should parse correctly with proper precedence
        assert!(program.items.len() == 1);
    }

    #[test]
    fn test_parse_boolean_expressions() {
        let source = r#"
        main {
            bool v$0 = true;
            bool v$1 = false;
            bool v$2 = v$0 && v$1;
            bool v$3 = !v$0;
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::MainBlock(main) => {
                assert_eq!(main.body.statements.len(), 4);
            }
            _ => panic!("Expected main block"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = r#"
        main {
            bool v$0 = true;
            if (v$0) {
                int v$1 = 42;
            } else {
                int v$2 = 0;
            }
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::MainBlock(main) => {
                assert_eq!(main.body.statements.len(), 2);
                match &main.body.statements[1] {
                    Statement::If(if_stmt) => {
                        assert!(if_stmt.else_block.is_some());
                    }
                    _ => panic!("Expected if statement"),
                }
            }
            _ => panic!("Expected main block"),
        }
    }

    #[test]
    fn test_parse_comparison_operators() {
        let source = r#"
        main {
            int v$0 = 5;
            int v$1 = 10;
            bool v$2 = v$0 < v$1;
            bool v$3 = v$0 >= v$1;
            bool v$4 = v$0 == v$1;
            bool v$5 = v$0 != v$1;
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::MainBlock(main) => {
                assert_eq!(main.body.statements.len(), 6);
            }
            _ => panic!("Expected main block"),
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        let source = r#"
        main {
            bool v$0 = true;
            bool v$1 = false;
            bool v$2 = v$0 && v$1;
            bool v$3 = v$0 || v$1;
            bool v$4 = !v$0;
            bool v$5 = v$0 && !v$1;
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::MainBlock(main) => {
                assert_eq!(main.body.statements.len(), 6);
            }
            _ => panic!("Expected main block"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let source = r#"
        main {
            bool v$0 = true || false && true;  // Should be: true || (false && true)
            bool v$1 = !true && false;         // Should be: (!true) && false
            bool v$2 = 1 + 2 < 3 * 4;         // Should be: (1 + 2) < (3 * 4)
        }
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut parser = Parser::new(source, &mut symbol_table).unwrap();
        let program = parser.parse_program().unwrap();

        // Should parse without errors with correct precedence
        assert_eq!(program.items.len(), 1);
    }
}
