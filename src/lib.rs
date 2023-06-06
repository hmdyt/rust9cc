pub mod token {
    use std::iter::Peekable;

    fn is_digit<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<bool>{
        if let Some(i) = iter.peek() {
            Some(i.is_digit(10))
        } else {
            None
        }
    }

    pub fn str_to_u<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Option<u32> {
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
    }
}
