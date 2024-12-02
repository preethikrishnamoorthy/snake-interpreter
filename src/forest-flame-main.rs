use std::env;
use std::fs::File;
use std::io::prelude::*;

mod compile;
use forest_flame::utils;
use utils::Type;
use compile::compile;

use forest_flame::grammar::ProgramParser;
use forest_flame::lexer::Lexer;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let in_name = &args[1];
    let out_name = &args[2];

    // You will make result hold the result of actually compiling
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let lexer = Lexer::new(&in_contents);
    let parser = ProgramParser::new();
    let ast = parser.parse(lexer);

    match ast {
        Err(e) => panic!("Invalid: could not parse program!\nError msg: {}", e),
        Ok(_) => {}
    }

    let program = ast.unwrap();
    let (result, extend_stack, type_result) = compile(&program);
    
    let mut type_val = 0;
    match type_result {
        Type::Bool => type_val = 1,
        Type::Pointer(_) => type_val = 2,
        _ => {}
    }

    let asm_program = format!(
        "
section .bss
heap resq 1000000

section .text
extern snek_print
extern snek_error

global our_code_starts_here
{}

 mov rdi, rax    ; First argument: value to print
 mov rsi, {}      ; Second argument: type flag (0 for number)
 {}
 call snek_print
 leave 
ret
",
        result, type_val, extend_stack
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}
