use std::io::{self, Write};

use crate::ast::Node;

pub fn prefix<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, ".intel_syntax noprefix")?;
    writeln!(w, ".globl main")?;
    writeln!(w, "main:")?;
    Ok(())
}

pub fn suffix<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "  pop rax")?;
    writeln!(w, "  ret")?;
    Ok(())
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
    }
}
