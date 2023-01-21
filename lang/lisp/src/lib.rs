use std::collections::HashMap;

pub enum Node {
	Boolean(bool),
	Expression(Expr),
	Integer(i64),
	Keyword(Keyword),
	Nil,
	Real(f64),
	Symbol(String),
	Text(String)
}

pub struct Expr {
	pub op: Node,
	pub lhs: Node,
	pub rhs: Node,
}

#[repr(u8)]
pub enum Keyword {
	Block,
	If,
	Lambda,
	Let,
	Quote,
}

pub type SymbolMap = HashMap<String, Node>;

pub struct State {
	pub map: SymbolMap,
	pub exprs: Vec<Expr>,
	pub src: String,
}

impl State {
}
