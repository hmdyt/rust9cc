use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("引数の数が正しくありません");
    }

    let finalize_code = u8::from_str_radix(&args[1], 10).expect("u8を入力してください");

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", finalize_code);
    println!("  ret");
}
