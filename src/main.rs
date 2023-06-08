use std::env;
use std::io;

use rust9cc::{ast, gen, lexer};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("引数の数が正しくありません");
    }
    let mut c = args[1].chars().peekable();

    // tokenize
    let tokens = lexer::tokenize(&mut c);
    let mut token_iter = tokens.iter().peekable();

    // ast
    let node = ast::expr(&mut token_iter).unwrap();

    // gen assembly code to stdout
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    gen::prefix(&mut stdout)?;
    gen::from_node(&mut stdout, *node)?;
    gen::suffix(&mut stdout)?;

    Ok(())
}
