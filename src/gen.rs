use std::io::{self, Write};

use crate::ast::node::{LocalVar, Node, Nodes};

pub trait CodeGen<W: Write> {
    fn prefix(&mut self) -> io::Result<()>;
    fn prologue(&mut self) -> io::Result<()>;
    fn gen_from_nodes(&mut self, nodes: Nodes) -> io::Result<()>;
    fn epilogue(&mut self) -> io::Result<()>;
}

pub struct AsmCodeGen<W: Write> {
    w: W,
}

impl<W: Write> CodeGen<W> for AsmCodeGen<W> {
    fn prefix(&mut self) -> io::Result<()> {
        writeln!(self.w, ".intel_syntax noprefix")?;
        writeln!(self.w, ".globl main")?;
        writeln!(self.w, "main:")?;
        Ok(())
    }

    fn prologue(&mut self) -> io::Result<()> {
        writeln!(self.w, "  push rbp")?;
        writeln!(self.w, "  mov rbp, rsp")?;
        writeln!(self.w, "  sub rsp, 208")?;
        Ok(())
    }

    fn gen_from_nodes(&mut self, nodes: Nodes) -> io::Result<()> {
        for node in nodes.0 {
            self.from_node(*node)?;
            writeln!(self.w, "  pop rax")?;
        }
        Ok(())
    }

    fn epilogue(&mut self) -> io::Result<()> {
        writeln!(self.w, "  mov rsp, rbp")?;
        writeln!(self.w, "  pop rbp")?;
        writeln!(self.w, "  ret")?;
        Ok(())
    }
}

impl<W: Write> AsmCodeGen<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }

    fn lval(&mut self, node: Node) -> io::Result<()> {
        if let Node::Lvar(LocalVar { ident: _, offset }) = node {
            writeln!(self.w, "  mov rax, rbp")?;
            writeln!(self.w, "  sub rax, {}", offset)?;
            writeln!(self.w, "  push rax")?;
            Ok(())
        } else {
            panic!("代入の左辺値が変数ではありません");
        }
    }

    fn from_node(&mut self, node: Node) -> io::Result<()> {
        if let Node::Num(n) = node {
            writeln!(self.w, "  push {}", n)?;
            return Ok(());
        }

        match node {
            Node::Num(n) => {
                writeln!(self.w, "  push {}", n)?;
                Ok(())
            }
            Node::Add { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  add rax, rdi")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Sub { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  sub rax, rdi")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Mul { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  imul rax, rdi")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Div { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  cqo")?;
                writeln!(self.w, "  idiv rdi")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Eq { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  cmp rax, rdi")?;
                writeln!(self.w, "  sete al")?;
                writeln!(self.w, "  movzb rax, al")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Ne { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  cmp rax, rdi")?;
                writeln!(self.w, "  setne al")?;
                writeln!(self.w, "  movzb rax, al")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Lt { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  cmp rax, rdi")?;
                writeln!(self.w, "  setl al")?;
                writeln!(self.w, "  movzb rax, al")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Le { l, r } => {
                self.from_node(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  cmp rax, rdi")?;
                writeln!(self.w, "  setle al")?;
                writeln!(self.w, "  movzb rax, al")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Lvar(_) => {
                self.lval(node)?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  mov rax, [rax]")?;
                writeln!(self.w, "  push rax")?;
                Ok(())
            }
            Node::Assign { l, r } => {
                self.lval(*l)?;
                self.from_node(*r)?;
                writeln!(self.w, "  pop rdi")?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  mov [rax], rdi")?;
                writeln!(self.w, "  push rdi")?;
                Ok(())
            }
            Node::Return { expr } => {
                self.from_node(*expr)?;
                writeln!(self.w, "  pop rax")?;
                writeln!(self.w, "  mov rsp, rbp")?;
                writeln!(self.w, "  pop rbp")?;
                writeln!(self.w, "  ret")?;
                Ok(())
            }
        }
    }
}
