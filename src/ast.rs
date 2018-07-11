#[derive(PartialEq, Debug)]
pub enum Statement<E> {
    // Let binding
    SLet(String, E),
    // Probably side-effectual expression <expr>;
    SExpr(E),
    // TODO: Assignment?
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Id(String),
    Op(Box<Expr>, &'static str, Box<Expr>),
    Block(Vec<Statement<Expr>>, Option<Box<Expr>>)
}

#[derive(PartialEq, Debug)]
pub struct Function {
    pub name: String,
    pub argument_list: Vec<(String, String)>,
    pub body: Expr,
}
