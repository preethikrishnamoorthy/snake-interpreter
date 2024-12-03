use super::utils;
use dynasmrt::x64::Rq;
use im::HashMap;
use utils::Instr;
use utils::Op1;
use utils::Op2;
use utils::Reg;
use utils::Val;
use utils::Type;
use utils::Expr;
use utils::typecheck;

use std::sync::{LazyLock, Mutex};
use std::collections::HashSet;
use dynasmrt::{dynasm, DynasmApi};


static JUMP_LABEL: LazyLock<Mutex<i32>> = LazyLock::new(|| 0.into());

fn bool_to_num(b: &bool) -> i32 {
    match b {
        true => 1 as i32,
        false => 0 as i32
    }
}

pub fn compile_to_instrs(e: &Expr, stack_bindings: im::HashMap<String, i32>, 
    mut variable_types: &mut HashMap<String, Type>, stack_counter: i32, 
    defined_vars: &HashMap<String, i32>) -> Vec<Instr> {
    match e {
        Expr::Number(n) => return vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(*n))],
        Expr::Id(x) => {
            match stack_bindings.get(x) {
                None => {
                    match &mut defined_vars.get(x) {
                        Some (val) => {
                            return vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(**val))]
                        },
                        _ => panic!("Invalid: Unbound variable identifier {}", x),
                    }
                },
                Some(val) => return vec![Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RBP, -val))],
            }
        },
        Expr::UnOp(op, subexpr) => {
            let mut v = compile_to_instrs(subexpr, stack_bindings.clone(), variable_types,
                stack_counter, defined_vars);
            match op {
                Op1::Add1 => {
                    v.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(1)));
                    let mut current_jump = JUMP_LABEL.lock().unwrap();
                    v.push(Instr::Jno("unopAdd1Success".to_string(), *current_jump));
                    v.push(Instr::IMov(Val::Reg(Reg::RDI), Val::Imm(1)));
                    v.push(Instr::CallSnekErr());
                    v.push(Instr::Label("unopAdd1Success".to_string(), *current_jump));
                    *current_jump += 1;
                    drop(current_jump);
                }
                Op1::Sub1 => {
                    v.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(1)));
                    let mut current_jump = JUMP_LABEL.lock().unwrap();
                    v.push(Instr::Jno("unopSub1Success".to_string(), *current_jump));
                    v.push(Instr::IMov(Val::Reg(Reg::RDI), Val::Imm(1)));
                    v.push(Instr::CallSnekErr());
                    v.push(Instr::Label("unopSub1Success".to_string(), *current_jump));
                    *current_jump += 1;
                    drop(current_jump);
                }
            }
            return v;
        },
        Expr::BinOp(op, subexpr1, subexpr2) => {
            let mut v1 = compile_to_instrs(subexpr1, stack_bindings.clone(), &mut variable_types,
                stack_counter, defined_vars);
            // move first instructions to stack
            v1.push(Instr::Push(Val::Reg(Reg::RAX))); // -8
            let mut v2 = compile_to_instrs(subexpr2, stack_bindings.clone(), &mut variable_types,
                stack_counter + 8, defined_vars);
            v1.append(&mut v2);
            // move second instructions to RCX
            v1.push(Instr::IMov(Val::Reg(Reg::RCX), Val::Reg(Reg::RAX)));
            v1.push(Instr::Pop(Val::Reg(Reg::RAX))); // -8
            // move first instructions to rax
            
            match op {
                Op2::Plus => v1.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))), // -16
                Op2::Minus => v1.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))),
                Op2::Times => v1.push(Instr::IMul(Val::Reg(Reg::RAX), Val::Reg(Reg::RCX))),
                // need to add more instructions for the following comparison operations
                // cmp e1, e2 does e1 - e2
            }
            // let mut current_jump = JUMP_LABEL.lock().unwrap();
            // v1.push(Instr::Jno("binopSuccess".to_string(), *current_jump));
            // v1.push(Instr::IMov(Val::Reg(Reg::RDI), Val::Imm(1)));
            // v1.push(Instr::CallSnekErr());
            // v1.push(Instr::Label("binopSuccess".to_string(), *current_jump));
            // *current_jump += 1;
            // drop(current_jump);
            return v1;
        },
        Expr::Let(vec, e) => {
            let mut v = Vec::new(); 
            let mut items: HashSet<String> = HashSet::new();
            let mut mutable_copy = stack_bindings;
            let mut new_types = variable_types.clone();
            let mut new_scope_stack_counter = stack_counter;
            for item in vec {
                if items.contains(&item.0) {
                    panic!("Invalid: Duplicate binding");
                }
                items.insert(item.0.clone());
        
                let mut new_binding_expr = compile_to_instrs(&item.1, mutable_copy.clone(),
                &mut new_types, new_scope_stack_counter, defined_vars);
                
                new_types = new_types.update(item.0.clone(), typecheck(&item.1, &mut new_types.clone(), ));
                v.append(&mut new_binding_expr);

                mutable_copy = mutable_copy.update(item.0.clone(), new_scope_stack_counter);
                new_scope_stack_counter += 8;   
                
                v.push(Instr::Push(Val::Reg(Reg::RAX)));
            }
            v.append(&mut compile_to_instrs(e, mutable_copy.clone(), &mut new_types,
                new_scope_stack_counter, defined_vars));
            v.push(Instr::IAdd(Val::Reg(Reg::RSP), Val::Imm(8 * vec.len() as i32)));
            return v;
        },
        Expr::Set(var_name, e) => {
            let mut e_vec = compile_to_instrs(e, stack_bindings.clone(), &mut variable_types, stack_counter,
                defined_vars);
            match stack_bindings.get(var_name) {
                None => panic!("Invalid: Unbound variable identifier {}", var_name),
                Some(val) => {
                    // move new value into the spot on the stack where the old var value was stored
                    e_vec.push(Instr::IMov(Val::RegOffset(Reg::RBP, -val), Val::Reg(Reg::RAX)));
                    return e_vec;
                },
            }
        },
    }
}


fn reg_to_dynasm(r: &Reg) -> u8 {
    match r {
        Reg::R12 => Rq::R12 as u8,
        Reg::R13 => Rq::R13 as u8,
        Reg::R14 => Rq::R14 as u8,
        Reg::R15 => Rq::R15 as u8,
        Reg::RAX => Rq::RAX as u8,
        Reg::RBP => Rq::RBP as u8,
        Reg::RBX => Rq::RBX as u8,
        Reg::RDI => Rq::RDI as u8,
        Reg::RSI => Rq::RSI as u8,
        Reg::RSP => Rq::RSP as u8,
        Reg::RCX => Rq::RCX as u8,
    }
}

fn mov_to_asm(ops: &mut dynasmrt::x64::Assembler, dest: &Val, src: &Val) {
    match (dest, src) {
        (Val::Reg(dest_reg), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; mov Rq(reg_to_dynasm(dest_reg)), Rq(reg_to_dynasm(src_reg)));
        }
        (Val::Reg(dest_reg), Val::Imm(n)) => {
            dynasm!(ops; .arch x64; mov Rq(reg_to_dynasm(dest_reg)), *n);
        }
        (Val::Reg(dest_reg), Val::RegOffset(src_reg, offset)) => {
            dynasm!(ops; .arch x64; mov Rq(reg_to_dynasm(dest_reg)), [Rq(reg_to_dynasm(src_reg)) + *offset]);
        }
        (Val::RegOffset(dest_reg, offset), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; mov [Rq(reg_to_dynasm(dest_reg)) + *offset], Rq(reg_to_dynasm(src_reg)));
        }
        // (Val::RegOffset(dest_reg, offset), Val::Imm(n)) => {
        //     dynasm!(ops; .arch x64; mov [Rq(reg_to_dynasm(dest_reg)) + *offset], *n);
        // }
        _ => panic!("invalid mov"),
    }
}

fn add_to_asm(ops: &mut dynasmrt::x64::Assembler, dest: &Val, src: &Val) {
    match (dest, src) {
        (Val::Reg(dest_reg), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; add Rq(reg_to_dynasm(dest_reg)), Rq(reg_to_dynasm(src_reg)));
        }
        (Val::Reg(dest_reg), Val::Imm(n)) => {
            dynasm!(ops; .arch x64; add Rq(reg_to_dynasm(dest_reg)), *n);
        }
        (Val::Reg(dest_reg), Val::RegOffset(src_reg, offset)) => {
            dynasm!(ops; .arch x64; add Rq(reg_to_dynasm(dest_reg)), [Rq(reg_to_dynasm(src_reg)) + *offset]);
        }
        (Val::RegOffset(dest_reg, offset), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; add [Rq(reg_to_dynasm(dest_reg)) + *offset], Rq(reg_to_dynasm(src_reg)));
        }
        (Val::RegOffset(dest_reg, offset), Val::Imm(n)) => {
            dynasm!(ops; .arch x64; add [Rq(reg_to_dynasm(dest_reg)) + *offset], (*n).try_into().unwrap());
        }
        _ => panic!("invalid add"),
    }
}

fn sub_to_asm(ops: &mut dynasmrt::x64::Assembler, dest: &Val, src: &Val) {
    match (dest, src) {
        (Val::Reg(dest_reg), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; sub Rq(reg_to_dynasm(dest_reg)), Rq(reg_to_dynasm(src_reg)));
        }
        (Val::Reg(dest_reg), Val::Imm(n)) => {
            dynasm!(ops; .arch x64; sub Rq(reg_to_dynasm(dest_reg)), *n);
        }
        (Val::Reg(dest_reg), Val::RegOffset(src_reg, offset)) => {
            dynasm!(ops; .arch x64; sub Rq(reg_to_dynasm(dest_reg)), [Rq(reg_to_dynasm(src_reg)) + *offset]);
        }
        (Val::RegOffset(dest_reg, offset), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; sub [Rq(reg_to_dynasm(dest_reg)) + *offset], Rq(reg_to_dynasm(src_reg)));
        }
        (Val::RegOffset(dest_reg, offset), Val::Imm(n)) => {
            dynasm!(ops; .arch x64; sub [Rq(reg_to_dynasm(dest_reg)) + *offset], (*n).try_into().unwrap());
        }
        _ => panic!("invalid add"),
    }
}

fn mul_to_asm(ops: &mut dynasmrt::x64::Assembler, dest: &Val, src: &Val) {
    match (dest, src) {
        (Val::Reg(dest_reg), Val::Reg(src_reg)) => {
            dynasm!(ops; .arch x64; imul Rq(reg_to_dynasm(dest_reg)), Rq(reg_to_dynasm(src_reg)));
        }
        // (Val::Reg(dest_reg), Val::Imm(n)) => {
        //     dynasm!(ops; .arch x64; imul Rq(reg_to_dynasm(dest_reg)), *n);
        // }
        (Val::Reg(dest_reg), Val::RegOffset(src_reg, offset)) => {
            dynasm!(ops; .arch x64; imul Rq(reg_to_dynasm(dest_reg)), [Rq(reg_to_dynasm(src_reg)) + *offset]);
        }
        // (Val::RegOffset(dest_reg, offset), Val::Reg(src_reg)) => {
        //     dynasm!(ops; .arch x64; imul [Rq(reg_to_dynasm(dest_reg)) + *offset], Rq(reg_to_dynasm(src_reg)));
        // }
        // (Val::RegOffset(dest_reg, offset), Val::Imm(n)) => {
        //     dynasm!(ops; .arch x64; imul [Rq(reg_to_dynasm(dest_reg)) + *offset], *n);
        // }
        _ => panic!("invalid add"),
    }
}

fn pop_to_asm(ops: &mut dynasmrt::x64::Assembler, val: &Val) {
    match val {
        Val::Reg(r) => {
            dynasm!(ops; .arch x64; pop Rq(reg_to_dynasm(r)));
        }
        // Val::RegOffset(r, offset) => {
        //     dynasm!(ops; .arch x64; pop [Rq(reg_to_dynasm(src_reg)) + *offset]);
        // }
        _ => panic!("invalid pop"),
    }
}
fn push_to_asm(ops: &mut dynasmrt::x64::Assembler, val: &Val) {
    match val {
        Val::Reg(r) => {
            dynasm!(ops; .arch x64; push Rq(reg_to_dynasm(r)));
        }
        // Val::RegOffset(r, offset) => {
        //     dynasm!(ops; .arch x64; push [Rq(reg_to_dynasm(src_reg)) + *offset]);
        // }
        Val::Imm(n) => {
            dynasm!(ops; .arch x64; push *n);
        }
        _ => panic!("invalid pop"),
    }
}

fn instr_to_asm(i: &Instr, ops: &mut dynasmrt::x64::Assembler) {
    match i {
        Instr::IMov(dest, src) => mov_to_asm(ops, dest, src),
        Instr::IAdd(dest, src) => add_to_asm(ops, dest, src),
        Instr::ISub(dest, src) => sub_to_asm(ops, dest, src),
        Instr::IMul(dest, src) => mul_to_asm(ops, dest, src),
        Instr::Pop(val) => pop_to_asm(ops, val),
        Instr::Push(val) => push_to_asm(ops, val),
        // Instr::Label(label_name, label_num) => {
        //     let full_label_name = label_name.to_string() + &label_num.to_string();
        //     dynasm!(ops; .arch x64; label:);
        // }
        _ => {
            panic!("Instruction not supported");
        }
    }
}

pub fn instrs_to_asm(cmds: &Vec<Instr>, ops: &mut dynasmrt::x64::Assembler) {
    cmds.iter().for_each(|c| instr_to_asm(c, ops))
}
