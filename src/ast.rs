use std::fmt;
use std::iter::Peekable;

use crate::lexer;
use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Node {
    Num(u32),
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

// expr = equality
pub fn expr<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    equality(tokens)
}

// equality = relational ("==" relational | "!=" relational)*
fn equality<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    let mut node = relational(tokens)?;
    loop {
        match tokens.peek() {
            Some(Token::Equal) => {
                tokens.next();
                let r = relational(tokens)?;
                node = Box::new(Node::Eq { l: node, r });
            }
            Some(Token::NotEqual) => {
                tokens.next();
                let r = relational(tokens)?;
                node = Box::new(Node::Ne { l: node, r });
            }
            _ => return Some(node),
        }
    }
}

// relational = add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    let mut node = add(tokens)?;
    loop {
        match tokens.peek() {
            Some(Token::LessThan) => {
                tokens.next();
                let r = add(tokens)?;
                node = Box::new(Node::Lt { l: node, r });
            }
            Some(Token::LessThanOrEqual) => {
                tokens.next();
                let r = add(tokens)?;
                node = Box::new(Node::Le { l: node, r });
            }
            Some(Token::GreaterThan) => {
                tokens.next();
                let r = add(tokens)?;
                node = Box::new(Node::Lt { l: r, r: node });
            }
            Some(Token::GreaterThanOrEqual) => {
                tokens.next();
                let r = add(tokens)?;
                node = Box::new(Node::Le { l: r, r: node });
            }
            _ => break,
        }
    }
    Some(node)
}

// add = mul ("+" mul | "-" mul)*
fn add<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    let mut node = mul(tokens)?;
    loop {
        match tokens.peek() {
            Some(Token::Plus) => {
                tokens.next();
                let r = mul(tokens)?;
                node = Box::new(Node::Add { l: node, r });
            }
            Some(Token::Minus) => {
                tokens.next();
                let r = mul(tokens)?;
                node = Box::new(Node::Sub { l: node, r });
            }
            _ => break,
        }
    }
    Some(node)
}

// mul = unary ("*" unary | "/" unary)*
fn mul<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    let mut node = unary(tokens)?;
    loop {
        match tokens.peek() {
            Some(Token::Multiply) => {
                tokens.next();
                let r = unary(tokens)?;
                node = Box::new(Node::Mul { l: node, r });
            }
            Some(Token::Divide) => {
                tokens.next();
                let r = unary(tokens)?;
                node = Box::new(Node::Div { l: node, r });
            }
            _ => break,
        }
    }
    Some(node)
}

// unary   = ("+" | "-")? primary
fn unary<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    match tokens.peek() {
        Some(Token::Plus) => {
            tokens.next();
            primary(tokens)
        }
        Some(Token::Minus) => {
            tokens.next();
            let node = primary(tokens)?;
            Some(Box::new(Node::Sub {
                l: Box::new(Node::Num(0)),
                r: node,
            }))
        }
        _ => primary(tokens),
    }
}

// primary = num | "(" expr ")"
fn primary<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    match tokens.peek() {
        // 次のトークンが"("なら、"(" expr ")"のはず
        Some(Token::LeftParen) => {
            tokens.next();
            let node = expr(tokens)?;
            lexer::consume(tokens, Token::RightParen)?;
            Some(node)
        }
        // そうでなければ数値のはず
        Some(Token::Num(n)) => {
            tokens.next();
            Some(Box::new(Node::Num(*n)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        struct Test {
            name: &'static str,
            input: Vec<Token>,
            expected: &'static str,
        }

        let tests = vec![
            Test {
                name: "1 + 2",
                input: vec![Token::Num(1), Token::Plus, Token::Num(2)],
                expected: "(1 + 2)",
            },
            Test {
                name: "1 + 2 - 3",
                input: vec![
                    Token::Num(1),
                    Token::Plus,
                    Token::Num(2),
                    Token::Minus,
                    Token::Num(3),
                ],
                expected: "((1 + 2) - 3)",
            },
            Test {
                name: "1 + 2 * 3 / 4",
                input: vec![
                    Token::Num(1),
                    Token::Plus,
                    Token::Num(2),
                    Token::Multiply,
                    Token::Num(3),
                    Token::Divide,
                    Token::Num(4),
                ],
                expected: "(1 + ((2 * 3) / 4))",
            },
            Test {
                name: "(1 + 2) * (3 - 4) + 1",
                input: vec![
                    Token::LeftParen,
                    Token::Num(1),
                    Token::Plus,
                    Token::Num(2),
                    Token::RightParen,
                    Token::Multiply,
                    Token::LeftParen,
                    Token::Num(3),
                    Token::Minus,
                    Token::Num(4),
                    Token::RightParen,
                    Token::Plus,
                    Token::Num(1),
                ],
                expected: "(((1 + 2) * (3 - 4)) + 1)",
            },
            Test {
                name: "-1 + 2",
                input: vec![Token::Minus, Token::Num(1), Token::Plus, Token::Num(2)],
                expected: "((0 - 1) + 2)",
            },
            Test {
                name: "+2 -1",
                input: vec![Token::Plus, Token::Num(2), Token::Minus, Token::Num(1)],
                expected: "(2 - 1)",
            },
            Test {
                name: "1 + 2 < 3 + 4 == 5 * 6 <= 7 * 8",
                input: vec![
                    Token::Num(1),
                    Token::Plus,
                    Token::Num(2),
                    Token::LessThan,
                    Token::Num(3),
                    Token::Plus,
                    Token::Num(4),
                    Token::Equal,
                    Token::Num(5),
                    Token::Multiply,
                    Token::Num(6),
                    Token::LessThanOrEqual,
                    Token::Num(7),
                    Token::Multiply,
                    Token::Num(8),
                ],
                expected: "(((1 + 2) < (3 + 4)) == ((5 * 6) <= (7 * 8)))",
            },
            Test {
                name: "1 > 2",
                input: vec![Token::Num(1), Token::GreaterThan, Token::Num(2)],
                expected :"(2 < 1)"
            },
            Test {
                name: "1 >= 2",
                input: vec![Token::Num(1), Token::GreaterThanOrEqual, Token::Num(2)],
                expected :"(2 <= 1)"
            }
        ];

        for t in tests {
            let mut token_iter = t.input.iter().peekable();
            assert_eq!(
                format!("{}", expr(&mut token_iter).unwrap()),
                t.expected,
                "Faild in the {}",
                t.name,
            );
        }
    }
}
