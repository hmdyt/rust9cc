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
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Num(n) => write!(f, "{}", n),
            Node::Add { l, r } => write!(f, "({} + {})", l, r),
            Node::Sub { l, r } => write!(f, "({} - {})", l, r),
            Node::Mul { l, r } => write!(f, "({} * {})", l, r),
            Node::Div { l, r } => write!(f, "({} / {})", l, r),
        }
    }
}

// expr = mul ("+" mul | "-" mul)*
pub fn expr<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
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

// mul = primary ("*" primary | "/" primary)*
fn mul<'a, T: Iterator<Item = &'a Token>>(tokens: &mut Peekable<T>) -> Option<Box<Node>> {
    let mut node = primary(tokens)?;
    loop {
        match tokens.peek() {
            Some(Token::Multiply) => {
                tokens.next();
                let r = primary(tokens)?;
                node = Box::new(Node::Mul { l: node, r });
            }
            Some(Token::Divide) => {
                tokens.next();
                let r = primary(tokens)?;
                node = Box::new(Node::Div { l: node, r });
            }
            _ => break,
        }
    }
    Some(node)
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
