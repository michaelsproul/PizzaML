#[derive(PartialEq, Debug)]
pub enum Statement {
    // Let binding
    SLet(String, Expr),
    // Probably side-effectual expression <expr>;
    SExpr(Expr),
    // TODO: Assignment?
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Id(String),
    Op(Box<Expr>, &'static str, Box<Expr>),
    // if <cond> then <e1> else <e2>
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    FnCall {
        function: String,
        args: Vec<Expr>,
    },
    Block(Vec<Statement>, Option<Box<Expr>>)
}

#[derive(PartialEq, Debug)]
pub struct Function {
    pub name: String,
    pub argument_list: Vec<(String, String)>,
    pub body: Expr,
}
