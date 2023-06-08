use std::{env, io::Result};

use rust9cc::lexer::{expect_number, tokenize, Token};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("引数の数が正しくありません");
    }
    let mut c = args[1].chars().peekable();
    let tokens = tokenize(&mut c);
    let mut token_iter = tokens.iter().peekable();

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // 最初は数値でなければならない
    println!("  mov rax, {}", expect_number(&mut token_iter).unwrap());

    while let Some(token) = token_iter.next() {
        match token {
            Token::Minus => println!("  sub rax, {}", expect_number(&mut token_iter).unwrap()),
            Token::Plus => println!("  add rax, {}", expect_number(&mut token_iter).unwrap()),
            _ => panic!("予期しないトークンです: {:?}", token),
        }
    }

    println!("  ret");
    Ok(())
}
