use std::env;
use std::io;

use rust9cc::gen::CodeGen;
use rust9cc::{ast, gen, lexer};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("引数の数が正しくありません");
    }
    let mut c = args[1].chars().peekable();

    // tokenize
    let tokens = lexer::tokenize(&mut c);
    let mut token_iter = tokens.iter();

    // ast
    let mut parser = ast::parser::Parser::new(&mut token_iter);
    let nodes = parser.parse().unwrap();

    // gen assembly code to stdout
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut gen = gen::AsmCodeGen::new(&mut stdout);

    gen.prefix()?;
    gen.prologue()?;
    gen.gen_from_nodes(nodes)?;
    gen.epilogue()?;

    Ok(())
}
