use std::io::{self, Write};

use crate::ast::Node;

pub fn prefix<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, ".intel_syntax noprefix")?;
    writeln!(w, ".globl main")?;
    writeln!(w, "main:")?;
    Ok(())
}

pub fn prologue<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "  push rbp")?;
    writeln!(w, "  mov rbp, rsp")?;
    writeln!(w, "  sub rsp, 208")?;
    Ok(())
}

pub fn epilogue<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "  mov rsp, rbp")?;
    writeln!(w, "  pop rbp")?;
    writeln!(w, "  ret")?;
    Ok(())
}

fn lval<W: Write>(w: &mut W, node: Node) -> io::Result<()> {
    if let Node::Lvar { ident: _, offset } = node {
        writeln!(w, "  mov rax, rbp")?;
        writeln!(w, "  sub rax, {}", offset)?;
        writeln!(w, "  push rax")?;
        Ok(())
    } else {
        panic!("代入の左辺値が変数ではありません");
    }
}

pub fn from_node<W: Write>(w: &mut W, node: Node) -> io::Result<()> {
    if let Node::Num(n) = node {
        writeln!(w, "  push {}", n)?;
        return Ok(());
    }

    match node {
        Node::Num(n) => {
            writeln!(w, "  push {}", n)?;
            Ok(())
        }
        Node::Add { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  add rax, rdi")?;
            writeln!(w, "  push rax")?;
            Ok(())
        }
        Node::Sub { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  sub rax, rdi")?;
            writeln!(w, "  push rax")?;
            Ok(())
        }
        Node::Mul { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  imul rax, rdi")?;
            writeln!(w, "  push rax")?;
            Ok(())
        }
        Node::Div { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  cqo")?;
            writeln!(w, "  idiv rdi")?;
            writeln!(w, "  push rax")?;
            Ok(())
        }
        Node::Eq { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  cmp rax, rdi")?;
            writeln!(w, "  sete al")?;
            writeln!(w, "  movzb rax, al")?;
            writeln!(w, "  push rax")?;
            Ok(())
        },
        Node::Ne { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  cmp rax, rdi")?;
            writeln!(w, "  setne al")?;
            writeln!(w, "  movzb rax, al")?;
            writeln!(w, "  push rax")?;
            Ok(())
        },
        Node::Lt { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  cmp rax, rdi")?;
            writeln!(w, "  setl al")?;
            writeln!(w, "  movzb rax, al")?;
            writeln!(w, "  push rax")?;
            Ok(())
        },
        Node::Le { l, r } => {
            from_node(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  cmp rax, rdi")?;
            writeln!(w, "  setle al")?;
            writeln!(w, "  movzb rax, al")?;
            writeln!(w, "  push rax")?;
            Ok(())
        },
        Node::Lvar { ident: _, offset: _ } => {
            lval(w, node)?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  mov rax, [rax]")?;
            writeln!(w, "  push rax")?;
            Ok(())
        },
        Node::Assign { l, r } => {
            lval(w, *l)?;
            from_node(w, *r)?;
            writeln!(w, "  pop rdi")?;
            writeln!(w, "  pop rax")?;
            writeln!(w, "  mov [rax], rdi")?;
            writeln!(w, "  push rdi")?;
            Ok(())
        },
    }
}
