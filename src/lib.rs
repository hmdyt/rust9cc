pub mod util {
    use std::iter::Peekable;

    pub fn str_to_u<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> u32 {
        let mut result: u32 = 0;
        while let Some(i) = iter.peek() {
            match i.to_digit(10) {
                Some(n) => result = 10*result + n,
                None => break,
            }
            iter.next();
        }
        result
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_str_to_u() {
            let mut c = "1".chars().peekable();
            assert_eq!(str_to_u(&mut c), 1_u32);

            let mut c = "12".chars().peekable();
            assert_eq!(str_to_u(&mut c), 12_u32);

            let mut c = "12a".chars().peekable();
            assert_eq!(str_to_u(&mut c), 12_u32);
            assert_eq!(c.next().unwrap(), 'a');
        }
    } 
}