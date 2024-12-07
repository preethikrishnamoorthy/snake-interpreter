use im::HashMap;
// use modular_index::next;
use piston_window::types::Color;
use piston_window::*;

use crate::utils::Type;

use super::lexer::Lexer;
use super::compile::{compile_to_instrs, instrs_to_asm};
use super::grammar::ExpressionParser;

use std::mem;
use dynasmrt::{dynasm, DynasmApi};

use super::drawing::{draw_block, draw_rectange, draw_program_line, draw_blocks_count, draw_text};
use rand::{thread_rng, Rng};
use super::snake::{Direction, Snake};

const FOOD_COLOR: Color = [0.90, 0.49, 0.13, 1.0];
const BORDER_COLOR: Color = [0.741, 0.765, 0.78, 1.0];
const GAMEOVER_COLOR: Color = [0.91, 0.30, 0.24, 0.5];

const MOVING_PERIOD: f64 = 0.2; // in second
const RESTART_TIME: f64 = 1.0; // in second

pub struct Food {
    food_x: i32,
    food_y: i32,
    instr: String,
}

pub struct Program {
    line: String,
    y_value: i32,
    result: Option<i32>,
}

pub struct Game {
    snake: Snake,

    // Food
    food_list: Vec<Food>,
    // Game Space
    window_start_x: i32,
    width: i32,
    height: i32,

    // Game state
    is_game_over: bool,
    // finished_prog_line: bool,
    // When the game is running, it represents the waiting time from the previous moving
    // When the game is over, it represents the waiting time from the end of the game
    waiting_time: f64,
    is_def_line: bool,
    prog_line: String,
    program: Vec<Program>,
    def_bindings: Vec<i32>,
    temp_bindings: Vec<i32>,
    brace_count: i32,
    paren_count: i32,
    is_var_binding: bool,
    id_just_eaten: bool,
    prog_print_x: i32,
    prog_print_y: i32

}

impl Game {
    pub fn new(start_x: i32, width: i32, height: i32) -> Game {
        let mut g = Game {
            snake: Snake::new(start_x + 2, 2),
            waiting_time: 0.0,
            // food_list: vec![Food {food_x: 5, food_y: 5, instr: "+".to_string()},
            //                 Food {food_x: 7, food_y: 3, instr: "-".to_string()},
            //                 Food {food_x: 2, food_y: 8, instr: "*".to_string()},
            //                 Food {food_x: 8, food_y: 5, instr: "def".to_string()},
            //                 Food {food_x: 6, food_y: 6, instr: ";".to_string()},
            // ],
            food_list: vec![],
            window_start_x: start_x,
            width: width,
            height: height,
            is_game_over: false,
            is_def_line: false,
            prog_line: "( ".to_string(),
            program: vec![],
            def_bindings: vec![],
            temp_bindings: vec![],
            brace_count: 0,
            paren_count: 0,
            is_var_binding: false,
            id_just_eaten: false,
            prog_print_x: start_x + width + 1,
            prog_print_y: 2
        };
        // make food list anything that could follow (
        g.update_food("(".to_string());
        g
    }

    pub fn key_pressed(&mut self, key: Key) {
        if self.is_game_over {
            return;
        }

        let dir = match key {
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            // Ignore other keys
            _ => return,
        };

        if dir.unwrap() == self.snake.head_direction().opposite() {
            return;
        }

        // Check if the snake hits the border
        self.update_snake(dir);
    }

    pub fn draw(&self, con: &Context, g: &mut G2d, mut font: &mut Glyphs) {
        self.snake.draw(con, g, &mut font);

        for food in &self.food_list {
            draw_block(Self::instr_to_color(food.instr.clone()), 
                &food.instr.clone(), food.food_x, food.food_y, con, g, font);
        }

        // Draw the border
        draw_rectange(BORDER_COLOR, self.window_start_x, 0, self.width, 1, con, g); // top
        draw_rectange(BORDER_COLOR, self.window_start_x, self.height - 1, self.width, 1, con, g); // bottom
        draw_rectange(BORDER_COLOR, self.window_start_x, 0, 1, self.height, con, g); // left
        draw_rectange(BORDER_COLOR, self.window_start_x + self.width - 1, 0, 1, self.height, con, g); // right

        draw_blocks_count(self.snake.blocks_traveled(), con, g, font);

        // draw heap variables and their values
        let heap_x = 20.0;
        let mut heap_y = 70.0;
        for (var_num, value) in self.def_bindings.clone().into_iter().enumerate() {
            let mut text_to_draw = "x".to_string();
            text_to_draw.push_str(&var_num.to_string());
            text_to_draw.push_str(": ");
            text_to_draw.push_str(&value.to_string());
            draw_text(text_to_draw, [1.0, 1.0, 1.0, 1.0], heap_x, heap_y, con, g, font);
            heap_y += 20.0;
        }

        let temp_x = 140.0;
        let mut temp_y = 70.0;
        // draw temp variables and their values
        for (var_num, value) in self.temp_bindings.clone().into_iter().enumerate() {
            let mut text_to_draw = "y".to_string();
            text_to_draw.push_str(&var_num.to_string());
            text_to_draw.push_str(": ");
            text_to_draw.push_str(&value.to_string());
            draw_text(text_to_draw, [1.0, 1.0, 1.0, 1.0], temp_x, temp_y, con, g, font);
            temp_y += 20.0;
        }

        for program in &self.program {
            draw_program_line(program.line.clone(), program.result, self.prog_print_x, program.y_value, con, g, font);
        }
        draw_program_line(self.prog_line.clone(), None, self.prog_print_x, self.prog_print_y, con, g, font);

        // Draw a game-over rectangle
        if self.is_game_over {
            draw_rectange(GAMEOVER_COLOR, self.window_start_x, 0, self.width, self.height, con, g);
        }
    }

    fn instr_to_color(instr: String) -> Color {
        // [red, green, blue, alpha]
        //
        // All values are between 0.0 and 1.0.
        // For example, black is `[0.0, 0.0, 0.0, 1.0]` and white is `[1.0, 1.0, 1.0, 1.0]`.
        match instr.as_str() {
            "+" => return [0.0, 1.0, 1.0, 1.0], // yellow
            "-" => return [1.0, 0.5, 0.0, 1.0], // orange
            "*" => return [1.0, 0.0, 0.0, 1.0], // red
            ";" => return [0.0, 1.0, 0.0, 1.0], // yellow green
            "def" => return [0.0, 1.0, 0.5, 1.0], // bluer green
            "add1" => return [0.0, 1.0, 1.0, 1.0], // cyan
            "sub1" => return [0.0, 0.5, 1.0, 1.0], // blue
            "let" => return [0.0, 0.0, 1.0, 1.0], // dark blue
            "set" => return [0.5, 0.0, 1.0, 1.0], // purplish blue
            "(" => return [1.0, 0.0, 1.0, 1.0], // pink
            ")" => return [0.5, 0.0, 0.5, 1.0], // purple
            "{" => return [0.5, 0.0, 0.0, 1.0], // maroon
            "}" => return [0.0, 0.5, 0.0, 1.0], // dark green
            "var" => return [0.0, 0.0, 0.5, 1.0], // navy blue
            ":=" => return [0.0, 0.5, 0.5, 1.0], // dark teal
            "id" => return [1.0, 0.5, 0.5, 1.0], // salmon
            // identifier case
            _ => return FOOD_COLOR, // orange?
        }
        
    }    

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;

        // If the game is over
        if self.is_game_over {
            if self.waiting_time > RESTART_TIME {
                self.restart();
            }
            return;
        }

        // Move the snake
        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }

    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        // let food_list_local ;
        let mut instr_eaten = "".to_string();
        for food in &self.food_list {
            if food.food_x == head_x && food.food_y == head_y { //ate food
                instr_eaten = food.instr.clone();
                break;
            }
        }
        
        if instr_eaten != "" {
            let next_num_instr = self.snake.blocks_traveled().to_string();
            self.snake.reset_blocks_traveled();

            match instr_eaten.as_str() {
                ";" => {
                    // add number of blocks moved by snake to program
                    self.prog_line.push_str(&next_num_instr);
                    // close any unclosed {
                    for _ in 0..self.brace_count {
                        self.prog_line.push_str("}");
                    }
                    // close paren in program
                    self.prog_line.push_str(&" )".to_string());
                    println!("{}", self.prog_line);
                    
                    if self.is_def_line {
                        println!("saving result to var number {}", self.def_bindings.len());
                    }

                    let res = self.run_line();

                    println!("res of running prev line: {}", res);

                    self.program.push(Program{
                        line: self.prog_line.clone(), 
                        y_value: self.prog_print_y, 
                        result: Some(res)
                    });

                    // move y to next line so that new prog line is printed below
                    self.prog_print_y += 1;

                    // save result of program line to heap or temp binding
                    if self.is_def_line {
                        self.def_bindings.push(res);
                    }
                    if self.is_var_binding {
                        self.temp_bindings.push(res);
                        self.is_var_binding = false;
                    }

                    // start new program line
                    self.prog_line = "( ".to_string();

                    self.is_def_line = false;
                    self.id_just_eaten = false;
                },
                "def" => self.is_def_line = true,
                "var" => self.is_var_binding = true,
                "let" => {
                    self.id_just_eaten = false;
                    self.prog_line.push_str(&" ".to_string());
                    self.prog_line.push_str(&instr_eaten.clone());
                    self.prog_line.push_str(&" { ".to_string());
                }
                "{" => {
                    self.brace_count += 1;
                    self.prog_line.push_str(&" { ".to_string());
                },
                // assuming that the let finished so temp bindings go out of scope
                "}" => {
                    self.id_just_eaten = false;
                    self.brace_count -= 1;
                    self.temp_bindings.clear();
                    self.prog_line.push_str(&" } ".to_string());
                },
                ")" => {
                    self.paren_count -= 1;
                    if ! self.id_just_eaten {
                        self.prog_line.push_str(&next_num_instr);
                        self.id_just_eaten = false;
                    }
                    self.prog_line.push_str(&" ".to_string());
                    self.prog_line.push_str(&instr_eaten.clone());
                    self.prog_line.push_str(&" ".to_string());
                },
                "+"| "-" | "*" => {
                    // if an existing var was eaten before this instr, don't add number of blocks moved to program
                    if ! self.id_just_eaten {
                        self.prog_line.push_str(&next_num_instr);
                        self.id_just_eaten = false;
                    }
                    self.prog_line.push_str(&" ".to_string());
                    self.prog_line.push_str(&instr_eaten.clone());
                    self.prog_line.push_str(&" ".to_string());
                },
                "(" => {
                    self.id_just_eaten = false;
                    self.paren_count += 1;
                    self.prog_line.push_str(&instr_eaten.clone());
                    self.prog_line.push_str(&" ".to_string());
                },
                ":=" | "add1" | "sub1" => {
                    self.id_just_eaten = false;
                    self.prog_line.push_str(&instr_eaten.clone());
                    self.prog_line.push_str(&" ".to_string());
                },
                _ => {
                    // just ate a heap variable
                    if instr_eaten.chars().next().unwrap() == 'x' {
                        self.id_just_eaten = true;
                        self.prog_line.push_str(&" ".to_string());
                        let var_num = instr_eaten.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
                        if var_num < self.def_bindings.len() {
                            self.prog_line.push_str(&self.def_bindings[var_num].to_string());
                        } else {
                            panic!("var {} not found in def bindings {}", instr_eaten, var_num)
                        }
                    }
                    // just ate a temp variable
                    else if instr_eaten.chars().next().unwrap() == 'y' {
                        self.id_just_eaten = true;
                        self.prog_line.push_str(&" ".to_string());
                        let var_num = instr_eaten.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
                        if var_num < self.temp_bindings.len() {
                            self.prog_line.push_str(&self.temp_bindings[var_num].to_string());
                        } else {
                            panic!("var {} not found in temp bindings {}", instr_eaten, var_num)
                        }
                    }
                    else {
                        println!("ate instr: {}", &instr_eaten.clone());
                        // self.prog_line.push_str(&next_num_instr);
                        self.prog_line.push_str(&" ".to_string());
                        self.prog_line.push_str(&instr_eaten.clone());
                        self.prog_line.push_str(&" ".to_string());
                    }
                }
            }
            
            self.update_food(instr_eaten.clone());
            self.snake.restore_last_removed();
        }
    }

    fn check_if_the_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head_position(dir);

        // Check if the snake hits itself
        if self.snake.is_overlap_except_tail(next_x, next_y) {
            return false;
        }

        // Check if the snake overlaps with the border
        next_x > self.window_start_x && next_y > 0 && next_x < self.width + self.window_start_x - 1 && next_y < self.height - 1
    }

    fn update_food(&mut self, last_instr: String) {
        let mut rng = thread_rng();
        let next_instrs = self.generate_instructions(last_instr);

        self.food_list = vec![];
        for instr in next_instrs {
            let mut new_x = rng.gen_range((self.window_start_x + 1)..(self.width - 1));
            let mut new_y = rng.gen_range(1..(self.height - 1));
            while self.snake.is_overlap_except_tail(new_x, new_y) {
                new_x = rng.gen_range((self.window_start_x + 1)..(self.width - 1));
                new_y = rng.gen_range(1..(self.height - 1));
            }

            let new_food = Food {
                food_x: new_x,
                food_y: new_y,
                instr: instr.to_string(),
            };
            self.food_list.push(new_food);
        }        
    }

    fn generate_instructions(&mut self, last_instr: String) -> Vec<String>{
        // return set of next possible tokens based on what can follow last_instr
        println!("is_def_line: {}", self.is_def_line);
        if self.is_def_line {
            return vec!["+".to_string(), "-".to_string(), "*".to_string(), ";".to_string()];
        } else {
            if last_instr.parse::<f64>().is_ok() { // instr is int
                return vec!["+".to_string(), "-".to_string(), "*".to_string(), ")".to_string(), ";".to_string()]
            }
            let mut var_names = vec![];
            // add identifier names from heap bindings
            for idx in 0..self.def_bindings.len() {
                let mut var_name = "x".to_string();
                var_name.push_str(&idx.to_string());
                var_names.push(var_name);
            }
            // add identifier names from temporary let bindings
            for idx in 0..self.temp_bindings.len() {
                let mut var_name = "y".to_string();
                var_name.push_str(&idx.to_string());
                var_names.push(var_name);
            }
            let int_follow_set = vec!["+".to_string(), "-".to_string(), "*".to_string(), ")".to_string(), ";".to_string()];

            let mut next_instrs = vec!["def".to_string()];
            match last_instr.as_str() {
                "(" => {
                    next_instrs.append(&mut vec!["(".to_string(), ")".to_string(), "let".to_string(), "add1".to_string(), "sub1".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    if self.temp_bindings.len() > 0 || self.def_bindings.len() > 0 {
                        next_instrs.push("set".to_string());
                    }
                },
                ":=" => {
                    next_instrs.append(&mut vec!["(".to_string(), "let".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    if self.temp_bindings.len() > 0 || self.def_bindings.len() > 0 {
                        next_instrs.push("set".to_string());
                    }
                    next_instrs.append(&mut var_names);
                },
                ")" => next_instrs.append(&mut vec!["+".to_string(), "-".to_string(), "*".to_string(), ")".to_string(), ";".to_string(), "}".to_string()]),
                ";" => {
                    next_instrs.append(&mut vec!["var".to_string(), "let".to_string(), "}".to_string(), "(".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    if self.temp_bindings.len() > 0 || self.def_bindings.len() > 0 {
                        next_instrs.push("set".to_string());
                    }
                    next_instrs.append(&mut var_names);
                },
                "var" => next_instrs.push("id".to_string()), 
                "set" => next_instrs.append(&mut var_names),
                "let" | "{" => {
                    next_instrs.append(&mut vec!["let".to_string(), "(".to_string(), "var".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    if self.temp_bindings.len() > 0 || self.def_bindings.len() > 0 {
                        next_instrs.push("set".to_string());
                    }
                    next_instrs.append(&mut var_names);
                },
                "}" => {
                    next_instrs.append(&mut vec!["let".to_string(), "(".to_string(), "{".to_string(), "}".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    if self.temp_bindings.len() > 0 || self.def_bindings.len() > 0 {
                        next_instrs.push("set".to_string());
                    }
                    next_instrs.append(&mut var_names);
                }, 
                "+" => {
                    next_instrs.append(&mut vec!["(".to_string(), "add1".to_string(), "sub1".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    next_instrs.append(&mut var_names);
                },
                "-" => {
                    next_instrs.append(&mut vec!["(".to_string(), "add1".to_string(), "sub1".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    next_instrs.append(&mut var_names);
                }, 
                "*" => {
                    next_instrs.append(&mut vec!["(".to_string(), "add1".to_string(), "sub1".to_string()]);
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    next_instrs.append(&mut var_names);
                },
                "add1" => {
                    next_instrs.push("(".to_string());
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    next_instrs.append(&mut var_names);
                }, 
                "sub1" => {
                    next_instrs.push("(".to_string());
                    for s in int_follow_set {
                        if ! next_instrs.contains(&s) {
                            next_instrs.push(s);
                        }
                    }
                    next_instrs.append(&mut var_names);
                }, 
                "id" => {
                    next_instrs.push(":=".to_string());
                }, 
                // identifier case
                _ => next_instrs.append(&mut vec!["+".to_string(), "-".to_string(), "*".to_string(), ":=".to_string(), ")".to_string(), ";".to_string(), "}".to_string()]),
            }
            // remove ) from next instr if parens are balanced
            if self.paren_count == 0 {
                next_instrs.retain(|x| x != ")");
            }
            
            return next_instrs;
            // return vec!["+".to_string(), "-".to_string(), "*".to_string(), "def".to_string(), ";".to_string()];
        }
        
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.check_if_the_snake_alive(dir) {
            self.snake.move_forward(dir);
            self.check_eating();
        } else {
            self.is_game_over = true;
        }
        self.waiting_time = 0.0;
    }

    fn run_line(&mut self) -> i32 {
        let stack_bindings: HashMap<String, i32> = HashMap::new();
        let mut variable_types: HashMap<String, Type> = HashMap::new();
        let lexer = Lexer::new(&self.prog_line);
        let parser = ExpressionParser::new();
        let ast = parser.parse(lexer);
        
        
        match ast {
            Err(error_message) => {
                panic!("{}", error_message);
            }
            Ok(expression) => {
                println!("{:?}", expression);
                let mut ops = dynasmrt::x64::Assembler::new().unwrap();
                let start = ops.offset();
        
                let mut compilation_bindings = HashMap::new();
                for (idx, value) in self.def_bindings.clone().into_iter().enumerate() {
                    let mut var_name = "x".to_string();
                    var_name.push_str(&idx.to_string());
                    compilation_bindings.insert(var_name, value);
                }
                let instrs = compile_to_instrs(&expression, stack_bindings, &mut variable_types,
                    0, &compilation_bindings);
                println!("{:?}", instrs);
                instrs_to_asm(&instrs, &mut ops);
                dynasm!(ops
                ; .arch x64
                ; ret);
                let buf = ops.finalize().unwrap();
                let jitted_fn: extern "C" fn() -> i32 = unsafe { mem::transmute(buf.ptr(start)) };
    
                println!("{}", jitted_fn());
                return jitted_fn();
            }
        }
    }

    fn restart(&mut self) {
        self.snake = Snake::new(self.window_start_x + 2, 2);
        self.waiting_time = 0.0;
        self.update_food("(".to_string());
        self.is_game_over = false;
        self.prog_line = "( ".to_string();
        self.is_def_line = false;
        self.def_bindings = vec![];
        self.temp_bindings = vec![];
        // self.finished_prog_line = false;
    }
}
