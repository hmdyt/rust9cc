use std::{env, io::Result};

use rust9cc::util::str_to_u;

fn main() -> Result<()>{
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        panic!("引数の数が正しくありません");
    }
    let mut c = args[1].chars().peekable();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", str_to_u(&mut c));

    loop {
        match c.next() {
            Some('+')=> println!("  add rax, {}", str_to_u(&mut c)),
            Some('-') => println!("  sub rax, {}", str_to_u(&mut c)),
            Some(other) => panic!("予期しない文字です: {}", other),
            None => break,
        }
    }

    println!("  ret");

    Ok(())
}
