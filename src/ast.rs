use std::fmt;
use std::iter::Peekable;

use thiserror::Error;

use crate::lexer::Token;

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


#[derive(Debug, Error, PartialEq)]
pub enum ParserError {
    #[error("unexpected token: expected {expected:?}, actual {actual:?}")]
    UnexpectedToken {
        expected: Vec<Token>,
        actual: Vec<Token>,
    },
    #[error("unexpected EOF")]
    UnexpectedEOF,
    #[error("not enough tokens")]
    NotEnoughTokens,
}

type Result<T> = std::result::Result<T, ParserError>;

pub struct Parser<'a, T: Iterator<Item = &'a Token>> {
    tokens: Peekable<T>,
}

impl<'a, T: Iterator<Item = &'a Token>> Parser<'a, T> {
    pub fn new(tokens: T) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Nodes> {
        self.program()
    }

    fn consume(&mut self, token: Token) -> Result<()> {
        match self.tokens.peek() {
            Some(t) if **t == token => {
                self.tokens.next();
                Ok(())
            }
            Some(t) => Err(ParserError::UnexpectedToken {
                expected: vec![token],
                actual: vec![**t],
            }),
            None => Err(ParserError::NotEnoughTokens),
        }
    }

    fn peek(&mut self) -> Result<&Token> {
        match self.tokens.peek() {
            Some(t) => Ok(t),
            None => Err(ParserError::NotEnoughTokens),
        }
    }

    // program = stmt*
    fn program(&mut self) -> Result<Nodes> {
        let mut nodes = Vec::new();
        while self.consume(Token::EOF).is_err() {
            nodes.push(self.stmt()?);
        }
        Ok(Nodes(nodes))
    }

    // stmt = expr ";"
    fn stmt(&mut self) -> Result<Box<Node>> {
        let node = self.expr()?;
        self.consume(Token::Semicolon)?;
        Ok(node)
    }

    // expr = assign
    fn expr(&mut self) -> Result<Box<Node>> {
        self.assign()
    }

    // assign = equality ("=" assign)?
    fn assign(&mut self) -> Result<Box<Node>> {
        let mut node = self.equality()?;
        if let Ok(()) = self.consume(Token::Assign) {
            node = Box::new(Node::Assign {
                l: node,
                r: self.assign()?,
            });
        }
        Ok(node)
    }

    // equality = relational ("==" relational | "!=" relational)*
    fn equality(&mut self) -> Result<Box<Node>> {
        let mut node = self.relational()?;
        loop {
            match *self.peek()? {
                Token::Equal => {
                    self.consume(Token::Equal)?;
                    node = Box::new(Node::Eq {
                        l: node,
                        r: self.relational()?,
                    });
                }
                Token::NotEqual => {
                    self.consume(Token::NotEqual)?;
                    node = Box::new(Node::Ne {
                        l: node,
                        r: self.relational()?,
                    });
                }
                _ => break,
            }
        }
        Ok(node)
    }

    // relational = add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> Result<Box<Node>> {
        let mut node = self.add()?;
        loop {
            match *self.peek()? {
                Token::LessThan => {
                    self.consume(Token::LessThan)?;
                    node = Box::new(Node::Lt {
                        l: node,
                        r: self.add()?,
                    });
                }
                Token::LessThanOrEqual => {
                    self.consume(Token::LessThanOrEqual)?;
                    node = Box::new(Node::Le {
                        l: node,
                        r: self.add()?,
                    });
                }
                Token::GreaterThan => {
                    self.consume(Token::GreaterThan)?;
                    node = Box::new(Node::Lt {
                        l: self.add()?,
                        r: node,
                    });
                }
                Token::GreaterThanOrEqual => {
                    self.consume(Token::GreaterThanOrEqual)?;
                    node = Box::new(Node::Le {
                        l: self.add()?,
                        r: node,
                    });
                }
                _ => break,
            }
        }
        Ok(node)
    }

    // add = mul ("+" mul | "-" mul)*
    fn add(&mut self) -> Result<Box<Node>> {
        let mut node = self.mul()?;
        loop {
            match *self.peek()? {
                Token::Plus => {
                    self.consume(Token::Plus)?;
                    node = Box::new(Node::Add {
                        l: node,
                        r: self.mul()?,
                    });
                }
                Token::Minus => {
                    self.consume(Token::Minus)?;
                    node = Box::new(Node::Sub {
                        l: node,
                        r: self.mul()?,
                    });
                }
                _ => break,
            }
        }
        Ok(node)
    }

    // mul = unary ("*" unary | "/" unary)*
    fn mul(&mut self) -> Result<Box<Node>> {
        let mut node = self.unary()?;
        loop {
            match *self.peek()? {
                Token::Multiply => {
                    self.consume(Token::Multiply)?;
                    node = Box::new(Node::Mul {
                        l: node,
                        r: self.unary()?,
                    });
                }
                Token::Divide => {
                    self.consume(Token::Divide)?;
                    node = Box::new(Node::Div {
                        l: node,
                        r: self.unary()?,
                    });
                }
                _ => break,
            }
        }
        Ok(node)
    }

    // unary   = ("+" | "-")? primary
    fn unary(&mut self) -> Result<Box<Node>> {
        if let Ok(()) = self.consume(Token::Plus) {
            self.primary()
        } else if let Ok(()) = self.consume(Token::Minus) {
            Ok(Box::new(Node::Sub {
                l: Box::new(Node::Num(0)),
                r: self.primary()?,
            }))
        } else {
            self.primary()
        }
    }

    // primary = num | ident | "(" expr ")"
    fn primary(&mut self) -> Result<Box<Node>> {
        match *self.peek()? {
            Token::Num(n) => {
                self.consume(Token::Num(n))?;
                Ok(Box::new(Node::Num(n)))
            }
            Token::Identifier(c) => {
                self.consume(Token::Identifier(c))?;
                Ok(Box::new(Node::Lvar {
                    ident: c,
                    offset: (c as usize - 'a' as usize + 1) * 8,
                }))
            }
            Token::LeftParen => {
                self.consume(Token::LeftParen)?;
                let node = self.expr()?;
                self.consume(Token::RightParen)?;
                Ok(node)
            }
            _ => Err(ParserError::UnexpectedToken {
                expected: vec![Token::Num(0), Token::Identifier('a'), Token::LeftParen],
                actual: vec![*self.peek().unwrap()],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;

    use super::*;

    #[test]
    fn test_parser() {
        struct Test {
            success: bool,
            name : &'static str,
            input: &'static str,
            expected: Option<&'static str>,
            expected_error: Option<ParserError>,
        }

        let tests = vec![
            Test{
                success: true,
                name: "add",
                input: "1+2;",
                expected: Some("(1 + 2); "),
                expected_error: None,
            },
            Test{
                success: true,
                name: "sub",
                input: "1-2;",
                expected: Some("(1 - 2); "),
                expected_error: None,
            },
            Test{
                success: true,
                name: "mul",
                input: "1*2;",
                expected: Some("(1 * 2); "),
                expected_error: None,
            },
            Test{
                success: true,
                name: "div",
                input: "1/2;",
                expected: Some("(1 / 2); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "four arithmetic operations",
                input: "1+2*3-4/2;",
                expected: Some("((1 + (2 * 3)) - (4 / 2)); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "four arithmetic operations with parentheses",
                input: "(1+2)*(3-4)/2;",
                expected: Some("(((1 + 2) * (3 - 4)) / 2); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "unary plus",
                input: "+1-2;",
                expected: Some("(1 - 2); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "unary minus",
                input: "-1+2;",
                expected: Some("((0 - 1) + 2); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "equality 1",
                input: "1 > 2 == 3 < (4 != 5);",
                expected: Some("((2 < 1) == (3 < (4 != 5))); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "equality 2",
                input: "1 >= 2 == 3 <= (4 != 5);",
                expected: Some("((2 <= 1) == (3 <= (4 != 5))); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "identifier",
                input: "a+z;",
                expected: Some("(a[rbp-8] + z[rbp-208]); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "assignment",
                input: "a=1;",
                expected: Some("(a[rbp-8] = 1); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "multi statements",
                input: "a=1;b=2;c=3;",
                expected: Some("(a[rbp-8] = 1); (b[rbp-16] = 2); (c[rbp-24] = 3); "),
                expected_error: None,
            },
            Test {
                success: false,
                name: "unexpected token 1",
                input: "1+;",
                expected: None,
                expected_error: Some(ParserError::UnexpectedToken {
                    expected: vec![Token::Num(0), Token::Identifier('a'), Token::LeftParen],
                    actual: vec![Token::Semicolon],
                }),
            },
            Test {
                success: false,
                name: "unexpected token 2",
                input: "1+",
                expected: None,
                expected_error: Some(ParserError::UnexpectedToken {
                    expected: vec![Token::Num(0), Token::Identifier('a'), Token::LeftParen],
                    actual: vec![Token::EOF],
                }),
            }
        ];

        for t in tests {
            let mut c = t.input.chars().peekable();
            let tokens = tokenize(&mut c);
            let mut token_iter = tokens.iter();
            let mut parser = Parser::new(&mut token_iter);
            match parser.program() {
                Ok(nodes) if t.success => {
                    assert_eq!(nodes.to_string(), t.expected.unwrap());
                }
                Ok(nodes) if !t.success => {
                    assert_ne!(nodes.to_string(), t.expected.unwrap());
                }
                Err(e) if t.success => {
                    panic!("{}: unexpected error: {:?}", t.name, e);
                }
                Err(e) if !t.success => {
                    assert_eq!(e, t.expected_error.unwrap());
                }
                other => {
                    panic!("{}: unexpected error: {:?}", t.name, other);
                }
            }
        }
    }
}
