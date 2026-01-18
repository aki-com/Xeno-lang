/// AST ノード定義
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    IntArray,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        ty: Type,
        value: Expr,
    },
    ExprStmt(Expr),
    Return(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Array(Vec<Expr>),
    Var(String),
    DataRef(String),          // .name
    Call { name: String, args: Vec<Expr> },
    Index { target: Box<Expr>, index: Box<Expr> },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub return_expr: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct DataDef {
    pub entries: Vec<(String, i64)>,
}

#[derive(Debug, Clone)]
pub enum FileContent {
    Function(Function),
    Data(DataDef),
}
