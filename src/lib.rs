pub mod token {
    use std::iter::Peekable;

    #[derive(Debug, PartialEq)]
    pub enum Token {
        Num(u32),
        Plus,
        Minus,
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
            let mut c = "1+2".chars().peekable();
            assert_eq!(
                tokenize(&mut c),
                vec![Token::Num(1), Token::Plus, Token::Num(2)]
            );

            let mut c = "1 + 2".chars().peekable();
            assert_eq!(
                tokenize(&mut c),
                vec![Token::Num(1), Token::Plus, Token::Num(2)]
            );

            let mut c = "1 + 2 - 3".chars().peekable();
            assert_eq!(
                tokenize(&mut c),
                vec![
                    Token::Num(1),
                    Token::Plus,
                    Token::Num(2),
                    Token::Minus,
                    Token::Num(3)
                ]
            );
        }

        #[test]
        fn test_expect_number() {
            let tokens = vec![Token::Num(1), Token::Plus, Token::Num(2)];
            let mut token_iter = tokens.iter();

            assert_eq!(expect_number(&mut token_iter), Some(1_u32));
            assert_eq!(expect_number(&mut token_iter), None);
        }
    }
}
