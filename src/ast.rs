use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Program {
    pub module: Option<String>,
    pub functions: BTreeMap<String, FunctionDecl>,
    pub tests: Vec<TestDecl>,
}

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct FunctionParam {
    pub name: String,
    pub type_name: Option<String>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct TestDecl {
    pub name: String,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let {
        name: String,
        expr: Expr,
        line: usize,
    },
    Set {
        name: String,
        expr: Expr,
        line: usize,
    },
    Return {
        expr: Expr,
        line: usize,
    },
    ExprStmt {
        expr: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    For {
        name: String,
        iterable: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    Expect {
        expr: Expr,
        line: usize,
    },
    Match {
        target: Expr,
        cases: Vec<MatchCase>,
        line: usize,
    },
}

#[derive(Clone, Debug)]
pub struct MatchCase {
    pub variant: String,
    pub binding: Option<String>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(LiteralValue),
    Variable {
        name: String,
        line: usize,
    },
    Unary {
        op: String,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        line: usize,
    },
    FieldAccess {
        target: Box<Expr>,
        field: String,
        line: usize,
    },
    RecordLiteral {
        type_name: String,
        fields: Vec<(String, Expr)>,
        line: usize,
    },
    VariantLiteral {
        variant: String,
        value: Option<Box<Expr>>,
        line: usize,
    },
    Try {
        expr: Box<Expr>,
        line: usize,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
        line: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}
