use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct LocalVar {
    pub offset: usize,
    pub ident: Box<String>,
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Num(u32),
    Lvar(LocalVar),
    Assign { l: Box<Node>, r: Box<Node> },
    Add { l: Box<Node>, r: Box<Node> },
    Sub { l: Box<Node>, r: Box<Node> },
    Mul { l: Box<Node>, r: Box<Node> },
    Div { l: Box<Node>, r: Box<Node> },
    Lt { l: Box<Node>, r: Box<Node> },
    Le { l: Box<Node>, r: Box<Node> },
    Eq { l: Box<Node>, r: Box<Node> },
    Ne { l: Box<Node>, r: Box<Node> },
    Return { expr: Box<Node> },
    If {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
    },
    While {
        cond: Box<Node>,
        then: Box<Node>,
    },
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        step: Option<Box<Node>>,
        then: Box<Node>,
    },
    Block {
        stmts: Vec<Box<Node>>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Num(n) => write!(f, "{}", n),
            Node::Lvar(LocalVar { ident, offset }) => write!(f, "{}[rbp-{}]", ident, offset),
            Node::Assign { l, r } => write!(f, "({} = {})", l, r),
            Node::Add { l, r } => write!(f, "({} + {})", l, r),
            Node::Sub { l, r } => write!(f, "({} - {})", l, r),
            Node::Mul { l, r } => write!(f, "({} * {})", l, r),
            Node::Div { l, r } => write!(f, "({} / {})", l, r),
            Node::Lt { l, r } => write!(f, "({} < {})", l, r),
            Node::Le { l, r } => write!(f, "({} <= {})", l, r),
            Node::Eq { l, r } => write!(f, "({} == {})", l, r),
            Node::Ne { l, r } => write!(f, "({} != {})", l, r),
            Node::Return { expr } => write!(f, "(return {})", expr),
            Node::If { cond, then, els } => {
                if let Some(els) = els {
                    write!(f, "(if ({}) {} else {})", cond, then, els)
                } else {
                    write!(f, "(if ({}) {})", cond, then)
                }
            },
            Node::While { cond, then } => write!(f, "(while ({}) {})", cond, then),
            Node::For { init, cond, step, then } => {
                write!(f, "(for ({}; {}; {}) {})",
                    init.as_ref().map(|n| n.to_string()).unwrap_or("".to_string()),
                    cond.as_ref().map(|n| n.to_string()).unwrap_or("".to_string()),
                    step.as_ref().map(|n| n.to_string()).unwrap_or("".to_string()),
                    then,
                )
            },
            Node::Block { stmts } => {
                write!(f, "{{ ")?;
                for stmt in stmts.iter() {
                    write!(f, "{}; ", **stmt)?;
                }
                write!(f, "}}")
            },
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
