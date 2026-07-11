use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Char,
    Bool,
    String,
    Void,
    Struct(String),
    Result {
        ok_type: Box<Type>,
        err_type: Box<Type>,
    },
    Array {
        element_type: Box<Type>,
        size: usize,
    },
    Pointer(Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Char => write!(f, "char"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Result { ok_type, err_type } => write!(f, "Result<{}, {}>", ok_type, err_type),
            Type::Array { element_type, size } => write!(f, "{}[{}]", element_type, size),
            Type::Pointer(inner) => write!(f, "{}*", inner),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDef),
    Struct(StructDef),
    Import(ImportStmt),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub return_type: Type,
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Block,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub param_type: Type,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub field_type: Type,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub module_name: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VariableDecl {
        var_type: Type,
        name: String,
        initializer: Option<Expression>,
        line: usize,
        col: usize,
    },
    Assignment {
        target: Expression,
        value: Expression,
        line: usize,
        col: usize,
    },
    ExpressionStmt {
        expr: Expression,
        line: usize,
        col: usize,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<ElseClause>,
        line: usize,
        col: usize,
    },
    While {
        condition: Expression,
        body: Block,
        line: usize,
        col: usize,
    },
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Box<Statement>>,
        body: Block,
        line: usize,
        col: usize,
    },
    Switch {
        expression: Expression,
        cases: Vec<SwitchCase>,
        default_case: Option<Block>,
        line: usize,
        col: usize,
    },
    Return {
        value: Option<Expression>,
        line: usize,
        col: usize,
    },
    Break {
        line: usize,
        col: usize,
    },
    Continue {
        line: usize,
        col: usize,
    },
}

#[derive(Debug, Clone)]
pub enum ElseClause {
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Box<ElseClause>>,
    },
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub value: Expression,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // Identifier
    Identifier(String),

    // Binary operations
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // Unary operations
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expression>,
    },

    // Function call
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    // Method call
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },

    // Field access
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },

    // Array access
    ArrayAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    // Struct construction
    StructConstruction {
        name: String,
        fields: Vec<(String, Expression)>,
    },

    // ok() and error() for Result type
    OkExpr(Box<Expression>),
    ErrorExpr(Box<Expression>),

    // String interpolation
    StringInterpolation {
        parts: Vec<StringPart>,
    },

    // Parenthesized expression
    Parenthesized(Box<Expression>),

    // Assignment expression
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Text(String),
    Expression(Expression),
}

impl Expression {
    pub fn line(&self) -> usize {
        match self {
            Expression::IntLiteral(_) => 0,
            Expression::FloatLiteral(_) => 0,
            Expression::StringLiteral(_) => 0,
            Expression::CharLiteral(_) => 0,
            Expression::BoolLiteral(_) => 0,
            Expression::Identifier(_) => 0,
            Expression::BinaryOp { left, .. } => left.line(),
            Expression::UnaryOp { expr, .. } => expr.line(),
            Expression::FunctionCall { .. } => 0,
            Expression::MethodCall { object, .. } => object.line(),
            Expression::FieldAccess { object, .. } => object.line(),
            Expression::ArrayAccess { object, .. } => object.line(),
            Expression::StructConstruction { .. } => 0,
            Expression::OkExpr(_) => 0,
            Expression::ErrorExpr(_) => 0,
            Expression::StringInterpolation { .. } => 0,
            Expression::Parenthesized(expr) => expr.line(),
            Expression::Assignment { target, .. } => target.line(),
        }
    }

    pub fn col(&self) -> usize {
        match self {
            Expression::IntLiteral(_) => 0,
            Expression::FloatLiteral(_) => 0,
            Expression::StringLiteral(_) => 0,
            Expression::CharLiteral(_) => 0,
            Expression::BoolLiteral(_) => 0,
            Expression::Identifier(_) => 0,
            Expression::BinaryOp { left, .. } => left.col(),
            Expression::UnaryOp { expr, .. } => expr.col(),
            Expression::FunctionCall { .. } => 0,
            Expression::MethodCall { object, .. } => object.col(),
            Expression::FieldAccess { object, .. } => object.col(),
            Expression::ArrayAccess { object, .. } => object.col(),
            Expression::StructConstruction { .. } => 0,
            Expression::OkExpr(_) => 0,
            Expression::ErrorExpr(_) => 0,
            Expression::StringInterpolation { .. } => 0,
            Expression::Parenthesized(expr) => expr.col(),
            Expression::Assignment { target, .. } => target.col(),
        }
    }
}
