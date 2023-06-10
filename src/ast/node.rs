use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Node {
    Num(u32),
    Lvar { ident: char, offset: usize },
    Assign { l: Box<Node>, r: Box<Node> },
    Add { l: Box<Node>, r: Box<Node> },
    Sub { l: Box<Node>, r: Box<Node> },
    Mul { l: Box<Node>, r: Box<Node> },
    Div { l: Box<Node>, r: Box<Node> },
    Lt { l: Box<Node>, r: Box<Node> },
    Le { l: Box<Node>, r: Box<Node> },
    Eq { l: Box<Node>, r: Box<Node> },
    Ne { l: Box<Node>, r: Box<Node> },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Num(n) => write!(f, "{}", n),
            Node::Lvar { ident, offset } => write!(f, "{}[rbp-{}]", ident, offset),
            Node::Assign { l, r } => write!(f, "({} = {})", l, r),
            Node::Add { l, r } => write!(f, "({} + {})", l, r),
            Node::Sub { l, r } => write!(f, "({} - {})", l, r),
            Node::Mul { l, r } => write!(f, "({} * {})", l, r),
            Node::Div { l, r } => write!(f, "({} / {})", l, r),
            Node::Lt { l, r } => write!(f, "({} < {})", l, r),
            Node::Le { l, r } => write!(f, "({} <= {})", l, r),
            Node::Eq { l, r } => write!(f, "({} == {})", l, r),
            Node::Ne { l, r } => write!(f, "({} != {})", l, r),
        }
    }
}

#[derive(Debug)]
pub struct Nodes(pub Vec<Box<Node>>);

impl fmt::Display for Nodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for n in self.0.iter() {
            write!(f, "{}; ", **n)?;
        }
        Ok(())
    }
}
