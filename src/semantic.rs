use std::collections::HashMap;
use crate::ast::*;
use crate::error::{MinusError, Result};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,
}

pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    current_scope: usize,
    functions: HashMap<String, FunctionDef>,
    structs: HashMap<String, StructDef>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let global_scope = Scope {
            symbols: HashMap::new(),
            parent: None,
        };

        let mut analyzer = SemanticAnalyzer {
            scopes: vec![global_scope],
            current_scope: 0,
            functions: HashMap::new(),
            structs: HashMap::new(),
        };

        // Register built-in Print function
        analyzer.functions.insert("Print".to_string(), FunctionDef {
            return_type: Type::Void,
            name: "Print".to_string(),
            params: vec![Parameter {
                param_type: Type::Int,
                name: "value".to_string(),
            }],
            body: Block {
                statements: vec![],
                line: 0,
                col: 0,
            },
            line: 0,
            col: 0,
        });

        analyzer
    }

    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all function and struct definitions
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    if self.functions.contains_key(&func.name) {
                        return Err(MinusError::DuplicateFunction {
                            name: func.name.clone(),
                        });
                    }
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Item::Struct(def) => {
                    if self.structs.contains_key(&def.name) {
                        return Err(MinusError::DuplicateVariable {
                            name: def.name.clone(),
                        });
                    }
                    self.structs.insert(def.name.clone(), def.clone());
                }
                Item::Import(_) => {}
            }
        }

        // Check for main function
        if !self.functions.contains_key("main") {
            return Err(MinusError::MainNotFound);
        }

        // Second pass: analyze each function
        for item in &program.items {
            if let Item::Function(func) = item {
                self.analyze_function(func)?;
            }
        }

        Ok(())
    }

    fn analyze_function(&mut self, func: &FunctionDef) -> Result<()> {
        self.enter_scope();

        // Add parameters to scope
        for param in &func.params {
            self.define_symbol(Symbol {
                name: param.name.clone(),
                ty: param.param_type.clone(),
                is_mutable: false,
            })?;
        }

        // Analyze body
        let returns_value = self.analyze_block(&func.body, &func.return_type)?;

        // Check return type
        if func.return_type != Type::Void && !returns_value {
            return Err(MinusError::MissingReturn {
                ty: func.return_type.to_string(),
            });
        }

        self.exit_scope();
        Ok(())
    }

    fn analyze_block(&mut self, block: &Block, expected_return: &Type) -> Result<bool> {
        self.enter_scope();

        let mut returns_value = false;
        for stmt in &block.statements {
            if self.analyze_statement(stmt, expected_return)? {
                returns_value = true;
            }
        }

        self.exit_scope();
        Ok(returns_value)
    }

    fn analyze_statement(&mut self, stmt: &Statement, expected_return: &Type) -> Result<bool> {
        match stmt {
            Statement::VariableDecl {
                var_type,
                name,
                initializer,
                ..
            } => {
                if let Some(init) = initializer {
                    let init_type = self.analyze_expression(init)?;
                    if !self.types_compatible(var_type, &init_type) {
                        return Err(MinusError::TypeError {
                            message: format!(
                                "Cannot assign {} to variable of type {}",
                                init_type, var_type
                            ),
                        });
                    }
                }

                self.define_symbol(Symbol {
                    name: name.clone(),
                    ty: var_type.clone(),
                    is_mutable: true,
                })?;
                Ok(false)
            }
            Statement::Assignment { target, value, .. } => {
                let target_type = self.analyze_expression(target)?;
                let value_type = self.analyze_expression(value)?;

                if !self.types_compatible(&target_type, &value_type) {
                    return Err(MinusError::TypeError {
                        message: format!(
                            "Cannot assign {} to variable of type {}",
                            value_type, target_type
                        ),
                    });
                }

                Ok(false)
            }
            Statement::ExpressionStmt { expr, .. } => {
                self.analyze_expression(expr)?;
                Ok(false)
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond_type = self.analyze_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(MinusError::TypeError {
                        message: format!("If condition must be bool, got {}", cond_type),
                    });
                }

                let mut returns = self.analyze_block(then_block, expected_return)?;

                if let Some(else_clause) = else_block {
                    match else_clause {
                        ElseClause::Block(block) => {
                            if self.analyze_block(block, expected_return)? {
                                returns = true;
                            }
                        }
                        ElseClause::If {
                            condition,
                            then_block,
                            else_block,
                        } => {
                            let else_if_stmt = Statement::If {
                                condition: condition.clone(),
                                then_block: then_block.clone(),
                                else_block: else_block.clone().map(|b| *b),
                                line: 0,
                                col: 0,
                            };
                            if self.analyze_statement(&else_if_stmt, expected_return)? {
                                returns = true;
                            }
                        }
                    }
                }

                Ok(returns)
            }
            Statement::While {
                condition, body, ..
            } => {
                let cond_type = self.analyze_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(MinusError::TypeError {
                        message: format!("While condition must be bool, got {}", cond_type),
                    });
                }

                self.analyze_block(body, expected_return)?;
                Ok(false)
            }
            Statement::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                self.enter_scope();

                if let Some(init_stmt) = init {
                    self.analyze_statement(init_stmt, expected_return)?;
                }

                if let Some(cond) = condition {
                    let cond_type = self.analyze_expression(cond)?;
                    if cond_type != Type::Bool {
                        return Err(MinusError::TypeError {
                            message: format!("For condition must be bool, got {}", cond_type),
                        });
                    }
                }

                if let Some(update_stmt) = update {
                    self.analyze_statement(update_stmt, expected_return)?;
                }

                self.analyze_block(body, expected_return)?;
                self.exit_scope();

                Ok(false)
            }
            Statement::Switch {
                expression,
                cases,
                default_case,
                ..
            } => {
                let switch_type = self.analyze_expression(expression)?;

                for case in cases {
                    let case_type = self.analyze_expression(&case.value)?;
                    if !self.types_compatible(&switch_type, &case_type) {
                        return Err(MinusError::TypeError {
                            message: format!(
                                "Case type {} doesn't match switch expression type {}",
                                case_type, switch_type
                            ),
                        });
                    }
                    self.analyze_block(&case.body, expected_return)?;
                }

                if let Some(default) = default_case {
                    self.analyze_block(default, expected_return)?;
                }

                Ok(false)
            }
            Statement::Return { value, .. } => {
                if let Some(return_value) = value {
                    let return_type = self.analyze_expression(return_value)?;
                    if !self.types_compatible(expected_return, &return_type) {
                        return Err(MinusError::TypeError {
                            message: format!(
                                "Return type {} doesn't match expected {}",
                                return_type, expected_return
                            ),
                        });
                    }
                    Ok(true)
                } else if *expected_return != Type::Void {
                    Err(MinusError::TypeError {
                        message: format!(
                            "Expected return value of type {}",
                            expected_return
                        ),
                    })
                } else {
                    Ok(true)
                }
            }
            Statement::Break { .. } | Statement::Continue { .. } => Ok(false),
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::IntLiteral(_) => Ok(Type::Int),
            Expression::FloatLiteral(_) => Ok(Type::Float),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::CharLiteral(_) => Ok(Type::Char),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::Identifier(name) => {
                if let Some(symbol) = self.lookup_symbol(name) {
                    Ok(symbol.ty.clone())
                } else {
                    Err(MinusError::UndefinedVariable {
                        name: name.clone(),
                    })
                }
            }
            Expression::BinaryOp { op, left, right } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if left_type == Type::Int && right_type == Type::Int {
                            Ok(Type::Int)
                        } else if left_type == Type::Float && right_type == Type::Float {
                            Ok(Type::Float)
                        } else if left_type == Type::String && right_type == Type::String && *op == BinaryOp::Add {
                            Ok(Type::String)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!(
                                    "Cannot apply {:?} to {} and {}",
                                    op, left_type, right_type
                                ),
                            })
                        }
                    }
                    BinaryOp::Modulo => {
                        if left_type == Type::Int && right_type == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!(
                                    "Modulo requires int operands, got {} and {}",
                                    left_type, right_type
                                ),
                            })
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!(
                                    "Cannot compare {} with {}",
                                    left_type, right_type
                                ),
                            })
                        }
                    }
                    BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual => {
                        if (left_type == Type::Int || left_type == Type::Float)
                            && (right_type == Type::Int || right_type == Type::Float)
                        {
                            Ok(Type::Bool)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!(
                                    "Cannot compare {} with {}",
                                    left_type, right_type
                                ),
                            })
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type == Type::Bool && right_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!(
                                    "Logical operator requires bool operands, got {} and {}",
                                    left_type, right_type
                                ),
                            })
                        }
                    }
                }
            }
            Expression::UnaryOp { op, expr } => {
                let expr_type = self.analyze_expression(expr)?;

                match op {
                    UnaryOp::Negate => {
                        if expr_type == Type::Int || expr_type == Type::Float {
                            Ok(expr_type)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!("Cannot negate {}", expr_type),
                            })
                        }
                    }
                    UnaryOp::Not => {
                        if expr_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(MinusError::TypeError {
                                message: format!("Cannot apply 'not' to {}", expr_type),
                            })
                        }
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                // Handle built-in Print function
                if name == "Print" {
                    if args.len() != 1 {
                        return Err(MinusError::TypeError {
                            message: format!(
                                "Print expects 1 argument, got {}",
                                args.len()
                            ),
                        });
                    }
                    self.analyze_expression(&args[0])?;
                    return Ok(Type::Void);
                }

                if let Some(func) = self.functions.get(name).cloned() {
                    if args.len() != func.params.len() {
                        return Err(MinusError::TypeError {
                            message: format!(
                                "Function {} expects {} arguments, got {}",
                                name,
                                func.params.len(),
                                args.len()
                            ),
                        });
                    }

                    for (arg, param) in args.iter().zip(func.params.iter()) {
                        let arg_type = self.analyze_expression(arg)?;
                        if !self.types_compatible(&param.param_type, &arg_type) {
                            return Err(MinusError::TypeError {
                                message: format!(
                                    "Argument type {} doesn't match parameter type {}",
                                    arg_type, param.param_type
                                ),
                            });
                        }
                    }

                    Ok(func.return_type.clone())
                } else {
                    Err(MinusError::UndefinedFunction {
                        name: name.clone(),
                    })
                }
            }
            Expression::MethodCall {
                object,
                method,
                args,
            } => {
                let object_type = self.analyze_expression(object)?;

                match (&object_type, method.as_str()) {
                    (Type::String, "len") => {
                        if !args.is_empty() {
                            return Err(MinusError::TypeError {
                                message: "String.len() takes no arguments".to_string(),
                            });
                        }
                        Ok(Type::Int)
                    }
                    (Type::String, "at") => {
                        if args.len() != 1 {
                            return Err(MinusError::TypeError {
                                message: "String.at() takes exactly one argument".to_string(),
                            });
                        }
                        let arg_type = self.analyze_expression(&args[0])?;
                        if arg_type != Type::Int {
                            return Err(MinusError::TypeError {
                                message: format!(
                                    "String.at() index must be int, got {}",
                                    arg_type
                                ),
                            });
                        }
                        Ok(Type::String)
                    }
                    _ => Err(MinusError::TypeError {
                        message: format!(
                            "No method '{}' for type {}",
                            method, object_type
                        ),
                    }),
                }
            }
            Expression::FieldAccess { object, field } => {
                let object_type = self.analyze_expression(object)?;

                if let Type::Struct(name) = &object_type {
                    if let Some(struct_def) = self.structs.get(name) {
                        if let Some(field_def) = struct_def.fields.iter().find(|f| f.name == *field) {
                            Ok(field_def.field_type.clone())
                        } else {
                            Err(MinusError::TypeError {
                                message: format!("Struct {} has no field '{}'", name, field),
                            })
                        }
                    } else {
                        Err(MinusError::TypeError {
                            message: format!("Unknown struct type: {}", name),
                        })
                    }
                } else {
                    Err(MinusError::TypeError {
                        message: format!(
                            "Cannot access field '{}' on type {}",
                            field, object_type
                        ),
                    })
                }
            }
            Expression::ArrayAccess { object, index } => {
                let object_type = self.analyze_expression(object)?;
                let index_type = self.analyze_expression(index)?;

                if index_type != Type::Int {
                    return Err(MinusError::TypeError {
                        message: format!("Array index must be int, got {}", index_type),
                    });
                }

                if let Type::Array { element_type, .. } = &object_type {
                    Ok(*element_type.clone())
                } else {
                    Err(MinusError::TypeError {
                        message: format!("Cannot index into type {}", object_type),
                    })
                }
            }
            Expression::StructConstruction { name, fields } => {
                if let Some(struct_def) = self.structs.get(name) {
                    for (field_name, _) in fields {
                        if !struct_def.fields.iter().any(|f| f.name == *field_name) {
                            return Err(MinusError::TypeError {
                                message: format!(
                                    "Struct {} has no field '{}'",
                                    name, field_name
                                ),
                            });
                        }
                    }
                    Ok(Type::Struct(name.clone()))
                } else {
                    Err(MinusError::TypeError {
                        message: format!("Unknown struct type: {}", name),
                    })
                }
            }
            Expression::OkExpr(expr) => {
                let inner_type = self.analyze_expression(expr)?;
                Ok(Type::Result {
                    ok_type: Box::new(inner_type),
                    err_type: Box::new(Type::String),
                })
            }
            Expression::ErrorExpr(expr) => {
                let inner_type = self.analyze_expression(expr)?;
                Ok(Type::Result {
                    ok_type: Box::new(Type::Void),
                    err_type: Box::new(inner_type),
                })
            }
            Expression::StringInterpolation { parts } => {
                for part in parts {
                    if let StringPart::Expression(expr) = part {
                        self.analyze_expression(expr)?;
                    }
                }
                Ok(Type::String)
            }
            Expression::Parenthesized(expr) => self.analyze_expression(expr),
            Expression::Assignment { target, value } => {
                let target_type = self.analyze_expression(target)?;
                let value_type = self.analyze_expression(value)?;
                if !self.types_compatible(&target_type, &value_type) {
                    return Err(MinusError::TypeError {
                        message: format!(
                            "Cannot assign {} to variable of type {}",
                            value_type, target_type
                        ),
                    });
                }
                Ok(target_type)
            }
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }

        // Allow implicit conversion between int and float in some cases
        matches!(
            (expected, actual),
            (Type::Int, Type::Float) | (Type::Float, Type::Int)
        )
    }

    fn enter_scope(&mut self) {
        let new_scope = Scope {
            symbols: HashMap::new(),
            parent: Some(self.current_scope),
        };
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }

    fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    fn define_symbol(&mut self, symbol: Symbol) -> Result<()> {
        if self.scopes[self.current_scope]
            .symbols
            .contains_key(&symbol.name)
        {
            return Err(MinusError::DuplicateVariable {
                name: symbol.name,
            });
        }

        self.scopes[self.current_scope]
            .symbols
            .insert(symbol.name.clone(), symbol);
        Ok(())
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = Some(self.current_scope);

        while let Some(idx) = scope_idx {
            if let Some(symbol) = self.scopes[idx].symbols.get(name) {
                return Some(symbol);
            }
            scope_idx = self.scopes[idx].parent;
        }

        None
    }
}
