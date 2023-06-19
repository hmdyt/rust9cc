use std::iter::Peekable;

use thiserror::Error;

use crate::ast::node::{LocalVar, Node, Nodes};
use crate::lexer::Token;

const LOCAL_VAR_OFFSET: usize = 8;

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
    locals: Vec<LocalVar>,
}

impl<'a, T: Iterator<Item = &'a Token>> Parser<'a, T> {
    pub fn new(tokens: T) -> Self {
        Parser {
            tokens: tokens.peekable(),
            locals: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Nodes> {
        self.program()
    }

    fn consume(&mut self, token: Token) -> Result<()> {
        // 引数にとるTokenが次のトークンと一致している時はトークンを消費してOk(())
        // 一致していない時トークンを消費せずErr(ParserError::UnexpectedToken)を返す
        match self.tokens.peek() {
            Some(t) if **t == token => {
                self.tokens.next();
                Ok(())
            }
            Some(t) => Err(ParserError::UnexpectedToken {
                expected: vec![token],
                actual: vec![(**t).clone()],
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

    fn get_local_var(&mut self, ident: &str) -> LocalVar {
        // 既に同じ名前の変数がある場合はそれを返す
        // なければ新しく作って返す
        // FIXME: cloneが多い
        // FIXME: identを探すのにO(n)かかる
        for var in &self.locals {
            if *var.ident == ident {
                return (*var).clone();
            }
        }

        let var = LocalVar {
            ident: Box::new(ident.to_string()),
            offset: (self.locals.len() + 1) * LOCAL_VAR_OFFSET,
        };
        self.locals.push(var);
        self.locals.last().unwrap().clone()
    }

    // program = stmt*
    fn program(&mut self) -> Result<Nodes> {
        let mut nodes = Vec::new();
        while self.consume(Token::EOF).is_err() {
            nodes.push(self.stmt()?);
        }
        Ok(Nodes(nodes))
    }

    // stmt = "{" stmt* "}"
    //      | "return" expr ";"
    //      | "if" "(" expr ")" stmt ("else" stmt)?
    //      | "while" "(" expr ")" stmt
    //      | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //      | expr ";"
    fn stmt(&mut self) -> Result<Box<Node>> {
        let next_token = self.peek()?.clone();
        match next_token {
            Token::LeftBrace => {
                self.consume(Token::LeftBrace)?;
                let mut stmts = Vec::new();
                while self.consume(Token::RightBrace).is_err() {
                    stmts.push(self.stmt()?);
                }
                Ok(Box::new(Node::Block { stmts }))
            }
            Token::Return => {
                self.consume(Token::Return)?;
                let node = Box::new(Node::Return { expr: self.expr()? });
                self.consume(Token::Semicolon)?;
                Ok(node)
            }
            Token::If => {
                self.consume(Token::If)?;
                self.consume(Token::LeftParen)?;
                let cond = self.expr()?;
                self.consume(Token::RightParen)?;
                let then = self.stmt()?;
                let els = if self.consume(Token::Else).is_ok() {
                    Some(self.stmt()?)
                } else {
                    None
                };
                Ok(Box::new(Node::If { cond, then, els }))
            }
            Token::While => {
                self.consume(Token::While)?;
                self.consume(Token::LeftParen)?;
                let cond = self.expr()?;
                self.consume(Token::RightParen)?;
                let then = self.stmt()?;
                Ok(Box::new(Node::While { cond, then }))
            }
            Token::For => {
                self.consume(Token::For)?;
                self.consume(Token::LeftParen)?;

                // expr? ";"
                let init = if self.consume(Token::Semicolon).is_ok() {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.consume(Token::Semicolon)?;

                // expr? ";"
                let cond = if self.consume(Token::Semicolon).is_ok() {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.consume(Token::Semicolon)?;

                // expr? ")"
                let step = if self.consume(Token::RightParen).is_ok() {
                    None
                } else {
                    Some(self.expr()?)
                };
                self.consume(Token::RightParen)?;

                let then = self.stmt()?;
                Ok(Box::new(Node::For {
                    init,
                    cond,
                    step,
                    then,
                }))
            }
            _ => {
                let node = self.expr()?;
                self.consume(Token::Semicolon)?;
                Ok(node)
            }
        }
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
        // FIXME: ここでcloneしているのが気持ち悪い
        // peekがmutableなのでだめ
        // Peekableの実装に乗っからずにtoken listを自前実装するのが良さそう
        let next_token = self.peek()?.clone();
        match next_token {
            Token::Num(n) => {
                self.consume(Token::Num(n))?;
                Ok(Box::new(Node::Num(n)))
            }
            Token::Identifier(s) => {
                self.consume(Token::Identifier(s.clone()))?;
                let var = self.get_local_var(&s);
                Ok(Box::new(Node::Lvar(var)))
            }
            Token::LeftParen => {
                self.consume(Token::LeftParen)?;
                let node = self.expr()?;
                self.consume(Token::RightParen)?;
                Ok(node)
            }
            _ => Err(ParserError::UnexpectedToken {
                expected: vec![Token::Num(0), Token::new_identifer("a"), Token::LeftParen],
                actual: vec![self.peek().unwrap().clone()],
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
            name: &'static str,
            input: &'static str,
            expected: Option<&'static str>,
            expected_error: Option<ParserError>,
        }

        let tests = vec![
            Test {
                success: true,
                name: "add",
                input: "1+2;",
                expected: Some("(1 + 2); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "sub",
                input: "1-2;",
                expected: Some("(1 - 2); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "mul",
                input: "1*2;",
                expected: Some("(1 * 2); "),
                expected_error: None,
            },
            Test {
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
                expected: Some("(a[rbp-8] + z[rbp-16]); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "assignment",
                input: "Ab123=1;",
                expected: Some("(Ab123[rbp-8] = 1); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "multi statements",
                input: "hoge=1;huga=2;piyo=3;",
                expected: Some("(hoge[rbp-8] = 1); (huga[rbp-16] = 2); (piyo[rbp-24] = 3); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "return",
                input: "returnx = 1;return returnx * 10;",
                expected: Some("(returnx[rbp-8] = 1); (return (returnx[rbp-8] * 10)); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "if",
                input: "x=1; if (x > 1) return 10*x;",
                expected: Some("(x[rbp-8] = 1); (if ((1 < x[rbp-8])) (return (10 * x[rbp-8]))); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "if else",
                input: "x=1;if (x > 1) return 10*x; else return 0;",
                expected: Some("(x[rbp-8] = 1); (if ((1 < x[rbp-8])) (return (10 * x[rbp-8])) else (return 0)); "),
                expected_error: None,
            },
            Test{
                success: true,
                name: "while",
                input: "x=1;while (x < 10) x = x + 1;",
                expected: Some("(x[rbp-8] = 1); (while ((x[rbp-8] < 10)) (x[rbp-8] = (x[rbp-8] + 1))); "),
                expected_error: None,
            },
            Test{
                success: true,
                name: "while with block",
                input: "x=1;while (x < 10) {x = x + 1; 1 + 2;}",
                expected: Some("(x[rbp-8] = 1); (while ((x[rbp-8] < 10)) { (x[rbp-8] = (x[rbp-8] + 1)); (1 + 2); }); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "for",
                input: "x=1;for (i=0;i<10;i=i+1) x = x + 1;",
                expected: Some("(x[rbp-8] = 1); (for ((i[rbp-16] = 0); (i[rbp-16] < 10); (i[rbp-16] = (i[rbp-16] + 1))) (x[rbp-8] = (x[rbp-8] + 1))); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "for with block",
                input: "x=1;for (i=0;i<10;i=i+1) {x = x + 1; 1 + 2;}",
                expected: Some("(x[rbp-8] = 1); (for ((i[rbp-16] = 0); (i[rbp-16] < 10); (i[rbp-16] = (i[rbp-16] + 1))) { (x[rbp-8] = (x[rbp-8] + 1)); (1 + 2); }); "),
                expected_error: None,
            },
            Test {
                success: true,
                name: "block",
                input: "{x=1;y=2;z=3;}",
                expected: Some("{ (x[rbp-8] = 1); (y[rbp-16] = 2); (z[rbp-24] = 3); }; "),
                expected_error: None,
            },
            Test {
                success: false,
                name: "unexpected token 1",
                input: "1+;",
                expected: None,
                expected_error: Some(ParserError::UnexpectedToken {
                    expected: vec![Token::Num(0), Token::new_identifer("a"), Token::LeftParen],
                    actual: vec![Token::Semicolon],
                }),
            },
            Test {
                success: false,
                name: "unexpected token 2",
                input: "1+",
                expected: None,
                expected_error: Some(ParserError::UnexpectedToken {
                    expected: vec![Token::Num(0), Token::new_identifer("a"), Token::LeftParen],
                    actual: vec![Token::EOF],
                }),
            },
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
