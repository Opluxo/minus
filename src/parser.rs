use crate::ast::*;
use crate::error::{MinusError, Result};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            let item = self.parse_item()?;
            items.push(item);
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item> {
        match self.peek() {
            Token::Import => Ok(Item::Import(self.parse_import()?)),
            Token::Struct => Ok(Item::Struct(self.parse_struct_def()?)),
            _ => Ok(Item::Function(self.parse_function_def()?)),
        }
    }

    fn parse_import(&mut self) -> Result<ImportStmt> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::Import)?;
        let module_name = self.expect_identifier()?;
        self.expect_token(Token::Semicolon)?;

        Ok(ImportStmt {
            module_name,
            line,
            col,
        })
    }

    fn parse_function_def(&mut self) -> Result<FunctionDef> {
        let line = self.current_line();
        let col = self.current_col();
        let return_type = self.parse_type()?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect_token(Token::RightParen)?;
        let body = self.parse_block()?;

        Ok(FunctionDef {
            return_type,
            name,
            params,
            body,
            line,
            col,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let param_type = self.parse_type()?;
                let name = self.expect_identifier()?;
                params.push(Parameter { param_type, name });

                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(params)
    }

    fn parse_struct_def(&mut self) -> Result<StructDef> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::Struct)?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::LeftBrace)?;
        let fields = self.parse_fields()?;
        self.expect_token(Token::RightBrace)?;

        Ok(StructDef {
            name,
            fields,
            line,
            col,
        })
    }

    fn parse_fields(&mut self) -> Result<Vec<Field>> {
        let mut fields = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let name = self.expect_identifier()?;
            self.expect_token(Token::Colon)?;
            let field_type = self.parse_type()?;

            fields.push(Field {
                field_type,
                name,
            });

            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        Ok(fields)
    }

    fn parse_block(&mut self) -> Result<Block> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::LeftBrace)?;
        let mut statements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect_token(Token::RightBrace)?;

        Ok(Block {
            statements,
            line,
            col,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Token::Int | Token::Float | Token::Char | Token::Bool | Token::String | Token::Void => {
                self.parse_variable_decl()
            }
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Switch => self.parse_switch(),
            Token::Return => self.parse_return(),
            Token::Break => {
                let line = self.current_line();
                let col = self.current_col();
                self.advance();
                self.expect_token(Token::Semicolon)?;
                Ok(Statement::Break { line, col })
            }
            Token::Continue => {
                let line = self.current_line();
                let col = self.current_col();
                self.advance();
                self.expect_token(Token::Semicolon)?;
                Ok(Statement::Continue { line, col })
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_decl(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        let var_type = self.parse_type()?;
        let name = self.expect_identifier()?;

        let initializer = if self.check(&Token::Assign) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_token(Token::Semicolon)?;

        Ok(Statement::VariableDecl {
            var_type,
            name,
            initializer,
            line,
            col,
        })
    }

    fn parse_if(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::If)?;
        self.expect_token(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        let then_block = self.parse_block()?;

        let else_block = if self.check(&Token::Else) {
            self.advance();
            if self.check(&Token::If) {
                Some(ElseClause::If {
                    condition: self.parse_expression()?,
                    then_block: self.parse_block()?,
                    else_block: None,
                })
            } else {
                Some(ElseClause::Block(self.parse_block()?))
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
            line,
            col,
        })
    }

    fn parse_while(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::While)?;
        self.expect_token(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        let body = self.parse_block()?;

        Ok(Statement::While {
            condition,
            body,
            line,
            col,
        })
    }

    fn parse_for(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::For)?;
        self.expect_token(Token::LeftParen)?;

        let init = if self.check(&Token::Semicolon) {
            self.advance();
            None
        } else if matches!(self.peek(), Token::Int | Token::Float | Token::Char | Token::Bool | Token::String) {
            Some(Box::new(self.parse_variable_decl()?))
        } else {
            Some(Box::new(self.parse_expression_statement()?))
        };

        let condition = if self.check(&Token::Semicolon) {
            self.advance();
            None
        } else {
            let expr = self.parse_expression()?;
            self.expect_token(Token::Semicolon)?;
            Some(expr)
        };

        let update = if self.check(&Token::RightParen) {
            None
        } else {
            let expr = self.parse_expression()?;
            Some(Box::new(Statement::ExpressionStmt {
                expr,
                line: self.current_line(),
                col: self.current_col(),
            }))
        };

        self.expect_token(Token::RightParen)?;
        let body = self.parse_block()?;

        Ok(Statement::For {
            init,
            condition,
            update,
            body,
            line,
            col,
        })
    }

    fn parse_switch(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::Switch)?;
        self.expect_token(Token::LeftParen)?;
        let expression = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        self.expect_token(Token::LeftBrace)?;

        let mut cases = Vec::new();
        let mut default_case = None;

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Case) {
                self.advance();
                let value = self.parse_expression()?;
                self.expect_token(Token::Colon)?;
                let mut stmts = Vec::new();

                while !self.check(&Token::Case) && !self.check(&Token::Default) && !self.check(&Token::RightBrace) {
                    stmts.push(self.parse_statement()?);
                }

                cases.push(SwitchCase {
                    value,
                    body: Block {
                        statements: stmts,
                        line: self.current_line(),
                        col: self.current_col(),
                    },
                });
            } else if self.check(&Token::Default) {
                self.advance();
                self.expect_token(Token::Colon)?;
                let mut stmts = Vec::new();

                while !self.check(&Token::Case) && !self.check(&Token::Default) && !self.check(&Token::RightBrace) {
                    stmts.push(self.parse_statement()?);
                }

                default_case = Some(Block {
                    statements: stmts,
                    line: self.current_line(),
                    col: self.current_col(),
                });
            } else {
                return Err(MinusError::ParseError {
                    line: self.current_line(),
                    col: self.current_col(),
                    message: "Expected 'case' or 'default' in switch statement".to_string(),
                });
            }
        }

        self.expect_token(Token::RightBrace)?;

        Ok(Statement::Switch {
            expression,
            cases,
            default_case,
            line,
            col,
        })
    }

    fn parse_return(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        self.expect_token(Token::Return)?;

        let value = if self.check(&Token::Semicolon) {
            self.advance();
            None
        } else {
            let expr = self.parse_expression()?;
            self.expect_token(Token::Semicolon)?;
            Some(expr)
        };

        Ok(Statement::Return { value, line, col })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let line = self.current_line();
        let col = self.current_col();
        let expr = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::ExpressionStmt {
            expr,
            line,
            col,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression> {
        let left = self.parse_or()?;

        if self.check(&Token::Assign) {
            self.advance();
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(left),
                value: Box::new(value),
            });
        }

        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expression> {
        let mut left = self.parse_and()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression> {
        let mut left = self.parse_equality()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;

        while self.check(&Token::Equal) || self.check(&Token::NotEqual) {
            let op = if self.check(&Token::Equal) {
                BinaryOp::Equal
            } else {
                BinaryOp::NotEqual
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive()?;

        while matches!(
            self.peek(),
            Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual
        ) {
            let op = match self.peek() {
                Token::Less => BinaryOp::Less,
                Token::Greater => BinaryOp::Greater,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative()?;

        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = if self.check(&Token::Plus) {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary()?;

        while matches!(
            self.peek(),
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = match self.peek() {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Negate,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Token::LeftParen => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect_token(Token::RightParen)?;

                    if let Expression::Identifier(name) = expr {
                        expr = Expression::FunctionCall {
                            name,
                            args,
                        };
                    } else {
                        return Err(MinusError::ParseError {
                            line: self.current_line(),
                            col: self.current_col(),
                            message: "Invalid function call".to_string(),
                        });
                    }
                }
                Token::Dot => {
                    self.advance();
                    let method = self.expect_identifier()?;

                    if self.check(&Token::LeftParen) {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect_token(Token::RightParen)?;

                        expr = Expression::MethodCall {
                            object: Box::new(expr),
                            method,
                            args,
                        };
                    } else {
                        expr = Expression::FieldAccess {
                            object: Box::new(expr),
                            field: method,
                        };
                    }
                }
                Token::LeftBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect_token(Token::RightBracket)?;

                    expr = Expression::ArrayAccess {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::IntLiteral(value) => {
                self.advance();
                Ok(Expression::IntLiteral(value))
            }
            Token::FloatLiteral(value) => {
                self.advance();
                Ok(Expression::FloatLiteral(value))
            }
            Token::StringLiteral(value) => {
                self.advance();
                Ok(Expression::StringLiteral(value))
            }
            Token::CharLiteral(value) => {
                self.advance();
                Ok(Expression::CharLiteral(value))
            }
            Token::BoolLiteral(value) => {
                self.advance();
                Ok(Expression::BoolLiteral(value))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(Expression::Parenthesized(Box::new(expr)))
            }
            Token::Ok => {
                self.advance();
                self.expect_token(Token::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(Expression::OkExpr(Box::new(expr)))
            }
            Token::Error => {
                self.advance();
                self.expect_token(Token::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(Expression::ErrorExpr(Box::new(expr)))
            }
            Token::StringInterpolationStart => {
                self.parse_string_interpolation()
            }
            Token::Print => {
                self.advance();
                self.expect_token(Token::LeftParen)?;
                let arg = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(Expression::FunctionCall {
                    name: "Print".to_string(),
                    args: vec![arg],
                })
            }
            _ => Err(MinusError::ParseError {
                line: self.current_line(),
                col: self.current_col(),
                message: format!("Unexpected token: {}", self.peek()),
            }),
        }
    }

    fn parse_string_interpolation(&mut self) -> Result<Expression> {
        self.advance(); // skip $"
        let mut parts = Vec::new();
        let mut current_text = String::new();

        loop {
            match self.peek() {
                Token::Eof => {
                    return Err(MinusError::ParseError {
                        line: self.current_line(),
                        col: self.current_col(),
                        message: "Unterminated string interpolation".to_string(),
                    });
                }
                Token::InterpolationEnd => {
                    if !current_text.is_empty() {
                        parts.push(StringPart::Text(current_text.clone()));
                        current_text.clear();
                    }
                    break;
                }
                _ => {
                    // Try to read text until { or end of string
                    // This is a simplified version - we'd need a proper tokenizer for interpolation
                    if let Token::InterpolationStart = self.peek() {
                        self.advance();
                        let expr = self.parse_expression()?;
                        self.expect_token(Token::InterpolationEnd)?;
                        parts.push(StringPart::Expression(expr));
                    } else {
                        // Consume as text
                        let token = self.advance();
                        current_text.push_str(&token.to_string());
                    }
                }
            }
        }

        Ok(Expression::StringInterpolation { parts })
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(args)
    }

    fn parse_type(&mut self) -> Result<Type> {
        let base_type = match self.peek().clone() {
            Token::Int => { self.advance(); Type::Int }
            Token::Float => { self.advance(); Type::Float }
            Token::Char => { self.advance(); Type::Char }
            Token::Bool => { self.advance(); Type::Bool }
            Token::String => { self.advance(); Type::String }
            Token::Void => { self.advance(); Type::Void }
            Token::Identifier(name) => {
                self.advance();
                Type::Struct(name)
            }
            Token::Result => {
                self.advance();
                self.expect_token(Token::Less)?;
                let ok_type = self.parse_type()?;
                self.expect_token(Token::Comma)?;
                let err_type = self.parse_type()?;
                self.expect_token(Token::Greater)?;
                return Ok(Type::Result {
                    ok_type: Box::new(ok_type),
                    err_type: Box::new(err_type),
                });
            }
            _ => {
                return Err(MinusError::ParseError {
                    line: self.current_line(),
                    col: self.current_col(),
                    message: format!("Expected type, got {}", self.peek()),
                });
            }
        };

        // Check for array or pointer
        if self.check(&Token::LeftBracket) {
            self.advance();
            let size = if let Token::IntLiteral(size) = self.peek() {
                let size = *size;
                self.advance();
                size as usize
            } else {
                return Err(MinusError::ParseError {
                    line: self.current_line(),
                    col: self.current_col(),
                    message: "Expected array size".to_string(),
                });
            };
            self.expect_token(Token::RightBracket)?;
            return Ok(Type::Array {
                element_type: Box::new(base_type),
                size,
            });
        }

        Ok(base_type)
    }

    fn expect_token(&mut self, expected: Token) -> Result<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(MinusError::ParseError {
                line: self.current_line(),
                col: self.current_col(),
                message: format!("Expected {}, got {}", expected, self.peek()),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match self.peek() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(MinusError::ParseError {
                line: self.current_line(),
                col: self.current_col(),
                message: format!("Expected identifier, got {}", self.peek()),
            }),
        }
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.peek() == &Token::Eof
    }

    fn current_line(&self) -> usize {
        0 // Would need to track this from lexer
    }

    fn current_col(&self) -> usize {
        0 // Would need to track this from lexer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_input(input: &str) -> Program {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }

    #[test]
    fn test_parse_function() {
        let input = "int add(int a, int b) { return a + b; }";
        let program = parse_input(input);

        assert_eq!(program.items.len(), 1);
        if let Item::Function(func) = &program.items[0] {
            assert_eq!(func.name, "add");
            assert_eq!(func.params.len(), 2);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_variable_decl() {
        let input = "void main() { int x = 5; }";
        let program = parse_input(input);

        assert_eq!(program.items.len(), 1);
        if let Item::Function(func) = &program.items[0] {
            assert_eq!(func.body.statements.len(), 1);
            if let Statement::VariableDecl { name, .. } = &func.body.statements[0] {
                assert_eq!(name, "x");
            }
        }
    }

    #[test]
    fn test_parse_if() {
        let input = "void main() { if (x > 0) { Print(x); } }";
        let program = parse_input(input);

        assert_eq!(program.items.len(), 1);
        if let Item::Function(func) = &program.items[0] {
            assert_eq!(func.body.statements.len(), 1);
            if let Statement::If { .. } = &func.body.statements[0] {
                // Success
            }
        }
    }
}
