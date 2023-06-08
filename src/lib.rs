pub mod token {
    use std::iter::Peekable;

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Num(u32),
        Plus,
        Minus,
        Multiply,
        Divide,
        LeftParen,  // "("
        RightParen, // ")"
    }

    pub fn tokenize<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(c) = iter.peek() {
            if c.is_whitespace() {
                iter.next();
                continue;
            }

            if c.is_digit(10) {
                tokens.push(Token::Num(str_to_u(iter).unwrap()));
                continue;
            }

            match iter.next() {
                Some('+') => tokens.push(Token::Plus),
                Some('-') => tokens.push(Token::Minus),
                Some('*') => tokens.push(Token::Multiply),
                Some('/') => tokens.push(Token::Divide),
                Some('(') => tokens.push(Token::LeftParen),
                Some(')') => tokens.push(Token::RightParen),
                Some(other) => panic!("予期しない文字です: {}", other),
                None => break,
            }
        }

        tokens
    }

    pub fn expect_number(tokens: &mut dyn Iterator<Item = &Token>) -> Option<u32> {
        if let Some(Token::Num(n)) = tokens.next() {
            Some(*n)
        } else {
            None
        }
    }

    pub fn consume<'a, T: Iterator<Item = &'a Token>>(
        tokens: &mut Peekable<T>,
        consuing_token: Token,
    ) -> Option<()> {
        match tokens.peek() {
            Some(token) if **token == consuing_token => {
                tokens.next();
                Some(())
            }
            _ => None,
        }
    }

    fn is_digit<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<bool> {
        if let Some(i) = iter.peek() {
            Some(i.is_digit(10))
        } else {
            None
        }
    }

    fn str_to_u<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<u32> {
        // 最初の文字が数字でなければNoneを返す
        if !is_digit(iter)? {
            return None;
        }

        let mut result: u32 = 0;
        while let Some(i) = iter.peek() {
            match i.to_digit(10) {
                Some(n) => result = 10 * result + n,
                None => break,
            }
            iter.next();
        }
        Some(result)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_str_to_u() {
            let mut c = "1".chars().peekable();
            assert_eq!(str_to_u(&mut c), Some(1_u32));

            let mut c = "12".chars().peekable();
            assert_eq!(str_to_u(&mut c), Some(12_u32));

            let mut c = "12a".chars().peekable();
            assert_eq!(str_to_u(&mut c), Some(12_u32));
            assert_eq!(c.next().unwrap(), 'a');

            let mut c = "a12".chars().peekable();
            assert_eq!(str_to_u(&mut c), None);
        }

        #[test]
        fn test_tokenize() {
            struct Test<'a> {
                name: &'a str,
                input: &'a str,
                expected: Vec<Token>,
            }

            let tests = vec![
                Test {
                    name: "1",
                    input: "1",
                    expected: vec![Token::Num(1)],
                },
                Test {
                    name: "1 + 2",
                    input: "1 + 2",
                    expected: vec![Token::Num(1), Token::Plus, Token::Num(2)],
                },
                Test {
                    name: "1 + 2 - 3",
                    input: "1 + 2 - 3",
                    expected: vec![
                        Token::Num(1),
                        Token::Plus,
                        Token::Num(2),
                        Token::Minus,
                        Token::Num(3),
                    ],
                },
                Test {
                    name: "カッコ",
                    input: "(1 + 2) - 3",
                    expected: vec![
                        Token::LeftParen,
                        Token::Num(1),
                        Token::Plus,
                        Token::Num(2),
                        Token::RightParen,
                        Token::Minus,
                        Token::Num(3),
                    ],
                },
                Test {
                    name: "四則演算",
                    input: "1 + 2 * (3 - 4) / 5",
                    expected: vec![
                        Token::Num(1),
                        Token::Plus,
                        Token::Num(2),
                        Token::Multiply,
                        Token::LeftParen,
                        Token::Num(3),
                        Token::Minus,
                        Token::Num(4),
                        Token::RightParen,
                        Token::Divide,
                        Token::Num(5),
                    ],
                },
            ];

            for t in tests {
                let mut c = t.input.chars().peekable();
                assert_eq!(tokenize(&mut c), t.expected, "Faild in the {}", t.name,);
            }
        }

        #[test]
        fn test_expect_number() {
            let tokens = vec![Token::Num(1), Token::Plus, Token::Num(2)];
            let mut token_iter = tokens.iter();

            assert_eq!(expect_number(&mut token_iter), Some(1_u32));
            assert_eq!(expect_number(&mut token_iter), None);
        }

        #[test]
        fn test_consume() {
            let tokens = vec![Token::LeftParen, Token::RightParen];
            let mut token_iter = tokens.iter().peekable();

            assert_eq!(consume(&mut token_iter, Token::LeftParen), Some(()));
            assert_eq!(token_iter.peek(), Some(&&Token::RightParen));

            assert_eq!(consume(&mut token_iter, Token::LeftParen), None);
            assert_eq!(token_iter.peek(), Some(&&Token::RightParen));

            assert_eq!(consume(&mut token_iter, Token::RightParen), Some(()));
            assert_eq!(token_iter.peek(), None);
        }
    }
}

pub mod ast {
    use std::fmt;
    use std::iter::Peekable;

    use crate::token;
    use crate::token::Token;

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
                token::consume(tokens, Token::RightParen)?;
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
}
