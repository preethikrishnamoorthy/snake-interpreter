use std::env;
use std::sync::{LazyLock, Mutex};

static IN_STRUCT: LazyLock<Mutex<bool>> = LazyLock::new(|| false.into());

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64) -> i64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    // TODO: print error message according to writeup
    if errcode == 1 {
        eprintln!("Integer overflow");
        std::process::exit(1);
    }
    eprintln!("an error ocurred {errcode}");
    std::process::exit(1);
}

#[export_name = "\x01snek_print"]
// 0 - int, 1 - bool, 2 - pointer, 3 - (, 4 - )
pub extern "C" fn snek_print(value: i64, type_flag: u64) {
    if type_flag == 1 {
        if value == 1 {
            print!("true ");
        }
        else {
            print!("false ");
        }
    }
    else if type_flag == 0 {
        print!("{} ", value);
    }
    else if type_flag == 2 {
        if value == 0 {
            print!("pointer: null ")
        }
        else {
            print!("pointer: {} ", value)
        }
        
    }

    let in_struct = IN_STRUCT.lock().unwrap();
    
    if !*in_struct {
        println!("");
    }

    drop(in_struct);


    
    if type_flag == 3 {
        let mut in_struct = IN_STRUCT.lock().unwrap();
        *in_struct = true;
        print!("(");
        drop(in_struct);
    }
    else if type_flag == 4 {
        let mut in_struct = IN_STRUCT.lock().unwrap();
        *in_struct = false;
        println!(")");
        drop(in_struct);
    }
}


fn parse_input(input: &str) -> i64 {
    if input.is_empty() {
        return 0;
    }
    return input.to_string().parse::<i64>().unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() == 2 { &args[1] } else { "0" };
    let input = parse_input(&input);

    let i: i64 = unsafe { our_code_starts_here(input) };
    println!("{i}");
}
