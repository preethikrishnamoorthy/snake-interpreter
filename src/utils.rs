use im::HashMap;
use core::panic;
use std::sync::LazyLock;

pub static KEYWORD_LIST : LazyLock<Vec<String>> =
 std::sync::LazyLock::new(
    || vec!["set!", "let", "if", "block", "true", "false", "add1", "sub1", "+", "-", "*", "input", "null"].into_iter().map(
        |s| s.to_string()
    ).collect()
 );

#[derive(Debug)]
pub enum Val {
    Reg(Reg),
    Imm(i32),
    RegOffset(Reg, i32),
    Str(String),
}

#[derive(Debug)]
pub enum Reg {
    RAX,
    RBP,
    RBX,
    RDI,
    RSI,
    RSP,
    RCX,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug)]
pub enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
    Label(String, i32),
    Jno(String, i32),
    CallSnekErr(),
    Pop(Val),
    Push(Val),
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Op1 {
    Add1,
    Sub1,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Op2 {
    Plus,
    Minus,
    Times,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
    Set(String, Box<Expr>),
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
pub enum Type {
    Int,
    Bool,
}

pub fn typecheck(e: &Expr, mut ctx: &mut HashMap<String, Type>) -> Type {
    match e {
        Expr::Number(_) => Type::Int,
        Expr::BinOp(_, e1, e2) => {
            let ty1 = typecheck(e1, &mut ctx);
            let ty2 = typecheck(e2, &mut ctx);
            if ty1 != ty2 {
                panic!("Invalid: type mismatch binop");             
            }
            return Type::Int;
            
        },
        Expr::Id(name) => {   
            match ctx.get(&name.to_string()) {
                Some(ty) => ty.clone(),
                None => panic!("Invalid: type error id {}", name),
            }
        },
        Expr::Let(bindings, body) => {
            for binding in bindings {
                if KEYWORD_LIST.contains(&binding.0) {
                    panic!("Invalid: variable name is a keyword");
                }
                let ty1 = typecheck(&binding.1, &mut ctx);
                *ctx = ctx.update(binding.0.clone(), ty1.clone());
            }
            typecheck(body, &mut ctx)
        },
        Expr::UnOp(op1, expr) => {
            let ty1 = typecheck(expr, &mut ctx);
            match op1 {
                _ => {
                    if ty1 != Type::Int {
                        panic!("Invalid: type mismatch: UnOp expects int");
                    }
                    Type::Int
                }
            }
        },
        Expr::Set(var_name, var_value) => {
            if KEYWORD_LIST.contains(var_name) {
                panic!("Invalid: variable name is a keyword");
            }
            typecheck(var_value, &mut ctx)
        },
    }
}


pub fn instr_to_str(i: &Instr) -> String {
    match i {
        Instr::IMov(val1, val2) => format!("mov {}, {}\n", val_to_str(val1), val_to_str(val2)),
        Instr::IAdd(val1, val2) => format!("add {}, {}\n", val_to_str(val1), val_to_str(val2)),
        Instr::ISub(val1, val2) => format!("sub {}, {}\n", val_to_str(val1), val_to_str(val2)),
        Instr::IMul(val1, val2) => format!("imul {}, {}\n", val_to_str(val1), val_to_str(val2)),
        Instr::Label(name, num) => format!("{}{}:\n", name.to_string(), num.to_string()),
        Instr::CallSnekErr() => format!("call snek_error\n"),
        Instr::Jno(name, num) => format!("jno {}{}\n", name.to_string(), num.to_string()),
        Instr::Pop(val1) => format!("pop {}\n", val_to_str(val1)),
        Instr::Push(val1) => format!("push {}\n", val_to_str(val1)),
    }
}

fn val_to_str(v: &Val) -> String {
    match v {
        Val::Imm(i) => i.to_string(),
        Val::Str(s) => s.to_string(),
        Val::Reg(Reg::RAX) => String::from("rax"),
        Val::Reg(Reg::RBP) => String::from("rbp"),
        Val::Reg(Reg::RSP) => String::from("rsp"),
        Val::Reg(Reg::RBX) => String::from("rbx"),
        Val::Reg(Reg::RCX) => String::from("rcx"),
        Val::Reg(Reg::R12) => String::from("r12"),
        Val::Reg(Reg::R13) => String::from("r13"),
        Val::Reg(Reg::R14) => String::from("r14"),
        Val::Reg(Reg::R15) => String::from("r15"),
        Val::Reg(Reg::RDI) => String::from("rdi"),
        Val::Reg(Reg::RSI) => String::from("rsi"),
        Val::RegOffset(Reg::RBP, i) => format!("[rbp + {}]", *i),
        Val::RegOffset(Reg::RSP, i) => format!("[rsp + {}]", *i),
        Val::RegOffset(Reg::RAX, i) => format!("[rax + {}]", *i),
        Val::RegOffset(Reg::RBX, i) => format!("[rbx + {}]", *i),
        Val::RegOffset(Reg::RDI, i) => format!("[rdi + {}]", *i),
        Val::RegOffset(Reg::RSI, i) => format!("[rsi + {}]", *i),
        Val::RegOffset(Reg::RCX, i) => format!("[rcx + {}]", *i),
        Val::RegOffset(Reg::R12, i) => format!("[r12 + {}]", *i),
        Val::RegOffset(Reg::R13, i) => format!("[r13 + {}]", *i),
        Val::RegOffset(Reg::R14, i) => format!("[r14 + {}]", *i),
        Val::RegOffset(Reg::R15, i) => format!("[r15 + {}]", *i),
    }
}

pub fn str_to_type(str: &String) -> Type {
    if str == "int" {
        return Type::Int;
    } else if str == "bool" {
        return Type::Bool;
    } else {
        panic!("invalid type");
    }
}

pub fn _type_to_str(t: Type) -> String {
    match t {
        Type::Bool => "bool".to_string(),
        Type::Int => "int".to_string(),
    }
}