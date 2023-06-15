use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Num(u32),
    Identifier(Box<String>),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,          // "("
    RightParen,         // ")"
    Equal,              // "=="
    NotEqual,           // "!="
    LessThan,           // "<"
    LessThanOrEqual,    // "<="
    GreaterThan,        // ">"
    GreaterThanOrEqual, // ">="
    Assign,             // "="
    Semicolon,          // ";"
    EOF,
}

impl Token {
    pub fn new_identifer(ident: &str) -> Token {
        Token::Identifier(Box::new(ident.to_string()))
    }
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
            Some(';') => tokens.push(Token::Semicolon),
            Some('=') => {
                if let Some('=') = iter.peek() {
                    iter.next();
                    tokens.push(Token::Equal);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            Some('!') => match iter.peek() {
                Some('=') => {
                    iter.next();
                    tokens.push(Token::NotEqual);
                }
                Some(other) => {
                    panic!("予期しない文字です: !{}", other);
                }
                _ => panic!("予期しない文字です: !"),
            },
            Some('<') => {
                if let Some('=') = iter.peek() {
                    iter.next();
                    tokens.push(Token::LessThanOrEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            Some('>') => {
                if let Some('=') = iter.peek() {
                    iter.next();
                    tokens.push(Token::GreaterThanOrEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            Some(alphabet) if 'a' <= alphabet && alphabet <= 'z' => {
                tokens.push(Token::new_identifer(&alphabet.to_string()));
            }
            Some(other) => panic!("予期しない文字です: {}", other),
            None => break,
        }
    }

    tokens.push(Token::EOF);
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
                expected: vec![Token::Num(1), Token::EOF],
            },
            Test {
                name: "1 + 2",
                input: "1 + 2",
                expected: vec![Token::Num(1), Token::Plus, Token::Num(2), Token::EOF],
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
                    Token::EOF,
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
                    Token::EOF,
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
                    Token::EOF,
                ],
            },
            Test {
                name: "比較演算子",
                input: "1 < 2 <= 3 > 4 >= 5 == 6 != 7",
                expected: vec![
                    Token::Num(1),
                    Token::LessThan,
                    Token::Num(2),
                    Token::LessThanOrEqual,
                    Token::Num(3),
                    Token::GreaterThan,
                    Token::Num(4),
                    Token::GreaterThanOrEqual,
                    Token::Num(5),
                    Token::Equal,
                    Token::Num(6),
                    Token::NotEqual,
                    Token::Num(7),
                    Token::EOF,
                ],
            },
            Test {
                name: "変数",
                input: "a + b - c",
                expected: vec![
                    Token::new_identifer("a"),
                    Token::Plus,
                    Token::new_identifer("b"),
                    Token::Minus,
                    Token::new_identifer("c"),
                    Token::EOF,
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
