use im::{HashMap, HashSet};
use lalrpop_util::ParseError;
use piston_window::types::Color;
use piston_window::*;

use super::utils::Instr;
use super::utils::Reg;
use super::utils::Val;

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
// const RESTART_TIME: f64 = 1.0; // in second

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

#[derive(Debug)]
pub enum GameState {
    StartScreen,
    GameStarted,
    SnakeDied,
    ReachedGoal
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
    reached_goal: bool,
    goal: i32,

    // When the game is running, it represents the waiting time from the previous moving
    // When the game is over, it represents the waiting time from the end of the game
    waiting_time: f64,

    is_def_line: bool,
    prog_line: String,
    program: Vec<Program>,

    def_bindings: Vec<i32>,
    num_let_bindings: i32,
    in_set: bool,
    in_let_binding: bool,

    prog_print_x: i32,
    prog_print_y: i32,

    last_instr: String,

    count_blocks: bool,
}

impl Game {
    pub fn new(start_x: i32, width: i32, height: i32, start_goal: i32) -> Game {
        let mut g = Game {
            snake: Snake::new(start_x + 2, 2),
            waiting_time: 0.0,
            food_list: vec![],
            window_start_x: start_x,
            width: width,
            height: height,
            is_game_over: false,
            is_def_line: false,
            prog_line: "( ".to_string(),
            program: vec![],
            def_bindings: vec![],
            prog_print_x: start_x + width + 1,
            prog_print_y: 4,
            reached_goal: false,
            goal: start_goal,
            num_let_bindings: 0,
            in_set: false,
            last_instr: "".to_string(),
            count_blocks: false,
            in_let_binding: false,

        };
        // make food list anything that could follow (
        g.update_food();
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
        for var_num in 0..self.num_let_bindings {
            let mut text_to_draw = "y".to_string();
            text_to_draw.push_str(&var_num.to_string());
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
        // All values are between 0.0 and 1.0.
        // For example, black is `[0.0, 0.0, 0.0, 1.0]` and white is `[1.0, 1.0, 1.0, 1.0]`.
        match instr.as_str() {
            "+" => return [0.0, 1.0, 1.0, 1.0], // yellow
            "-" => return [1.0, 0.5, 0.0, 1.0], // orange
            "*" => return [1.0, 0.0, 0.0, 1.0], // red
            "END" => return [0.0, 1.0, 0.0, 1.0], // yellow green
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
            "int" => return [0.27, 0.11, 0.39, 1.0], //grape
            "end_int" => return [0.27, 0.11, 0.39, 1.0], //grape
            "|" => return [0.91, 0.384, 0.384, 1.0], //pale red
            ";" => return [0.91, 0.384, 0.384, 1.0], //pale red
            // identifier case
            _ => return FOOD_COLOR, // orange?
        }
        
    }    

    pub fn update(&mut self, delta_time: f64) -> GameState {
        self.waiting_time += delta_time;

        let mut final_state = GameState::GameStarted;

        if self.is_game_over {
            final_state = GameState::SnakeDied
        } else if self.reached_goal {
            final_state = GameState::ReachedGoal
        }

        // If the game is over
        if self.is_game_over || self.reached_goal {
            self.restart();
            self.is_game_over = false;
            return final_state;
        }
        else {
            // Move the snake

            if self.waiting_time > MOVING_PERIOD {
                self.update_snake(None);
            }

            return final_state;
        }
    }

    fn update_prog(&mut self) {
        let instr_eaten = self.last_instr.clone();
        let next_num_instr = self.snake.blocks_traveled().to_string();
        self.snake.reset_blocks_traveled();

        match instr_eaten.as_str() {
            "int" => {
                self.count_blocks = true;
            }, //start of int blocks traveled
            "id" => {}, //start of display of id blocks
            "end_int" => { //end of int blocks traveled
                self.count_blocks = false;
                self.prog_line.push_str(&next_num_instr);
            },
            ";" => {
                println!("{}", self.prog_line);
                if self.is_def_line {
                    println!("saving result to var number {}", self.def_bindings.len());
                }

                let res = self.run_line();
                println!("res of running prev line: {}", res);

                if res == self.goal {
                    self.reached_goal = true;
                }

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

                // start new program line
                self.prog_line = "( ".to_string();

                self.is_def_line = false;
                self.num_let_bindings = 0;
                // self.snake = Snake::new(self.snake.head_position().0, self.snake.head_position().0); 
            },
            "def" => self.is_def_line = true,
            "var" => {
                let var_name = "y".to_owned() + &self.num_let_bindings.to_string();
                self.prog_line.push_str(&" var ".to_string());
                self.prog_line.push_str(&var_name);
                self.prog_line.push_str(&" := ".to_string());
            },
            "let" => {
                self.prog_line.push_str(&" let { ".to_string());
                self.in_let_binding = true;
            }
            "{" => {
                self.prog_line.push_str(&" { ".to_string());
            },
            // assuming that the let finished so temp bindings go out of scope
            "}" => {
                self.prog_line.push_str(&" } ".to_string());
                self.in_let_binding = false;
            },
            ")" => {
                self.prog_line.push_str(&" ".to_string());
                self.prog_line.push_str(&instr_eaten.clone());
                self.prog_line.push_str(&" ".to_string());
            },
            "+"| "-" | "*" => {
                // if an existing var was eaten before this instr, don't add number of blocks moved to program
                self.prog_line.push_str(&" ".to_string());
                self.prog_line.push_str(&instr_eaten.clone());
                self.prog_line.push_str(&" ".to_string());
            },
            "(" => {
                self.prog_line.push_str(&instr_eaten.clone());
                self.prog_line.push_str(&" ".to_string());
            },
            ":=" | "add1" | "sub1" => {
                self.prog_line.push_str(&instr_eaten.clone());
                self.prog_line.push_str(&" ".to_string());
                self.in_set = false; //once you define var to set
            },
            "set" => {
                println!("ate instr: {}", &instr_eaten.clone());
                self.prog_line.push_str(&" set ".to_string());
                self.in_set = true;
            },
            "|" => {
                println!("ate instr: {}", &instr_eaten.clone());
                self.prog_line.push_str(&" | ".to_string());
                self.num_let_bindings += 1;
            },
            _ => {
                // just ate a heap variable
                if instr_eaten.chars().next().unwrap() == 'x' {
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
                    self.prog_line.push_str(&" ".to_string());
                    self.prog_line.push_str(&instr_eaten);
                    self.prog_line.push_str(&" ");
                }
                else {
                    println!("ate instr: {}", &instr_eaten.clone());
                    self.prog_line.push_str(&" ".to_string());
                    self.prog_line.push_str(&instr_eaten.clone());
                    self.prog_line.push_str(&" ".to_string());
                }
            }
        }
    }

    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        let mut instr_eaten = "".to_string();
        for food in &self.food_list {
            if food.food_x == head_x && food.food_y == head_y { //ate food
                instr_eaten = food.instr.clone();
                break;
            }
        }
        
        if instr_eaten != "" { //ate something
            self.last_instr = instr_eaten.clone();
            self.update_prog();
            self.update_food();
            self.snake.restore_last_removed();
        }
    }

    fn generate_next_tokens(&mut self, last_instr: String) -> HashSet<String> {

        //ID CASE
        if last_instr == "id" {
            let mut var_names: HashSet<String> = HashSet::new();
            // add identifier names from heap bindings if we are not in a set expression
            if !self.in_set {
                for idx in 0..self.def_bindings.len() {
                    let mut var_name = "x".to_string();
                    var_name.push_str(&idx.to_string());
                    var_names.insert(var_name);
                }
            }
            
            // add identifier names from temporary let bindings
            for idx in 0..self.num_let_bindings {
                let mut var_name = "y".to_string();
                var_name.push_str(&idx.to_string());
                var_names.insert(var_name);
            }

            return var_names;
        }

        let mut tokens: HashSet<String> = HashSet::new(); //all tokens to return

        //INT CASE
        if last_instr == "int" {
            tokens.insert("end_int".to_string());
            return tokens;
        }

        if last_instr == "" || last_instr == ";" {  //first instr of line can be def, no other instr
            tokens.insert("def".to_string());
        }

        let lexer = Lexer::new(&self.prog_line);
        let parser = ExpressionParser::new();
        let ast = parser.parse(lexer);
        
        match ast {
            Err(error_message) => {
                match error_message {
                    ParseError::InvalidToken { location: _} => {
                        // println!("invalid token: {} | {}", location, self.prog_line);
                    },
                    ParseError::UnrecognizedEof { location: _, expected } => {
                        // println!("unrecognized EOF: {} | {}", location, self.prog_line);
                        tokens.extend(expected.iter().cloned());
                    },
                    ParseError::UnrecognizedToken { token: _, expected } => {
                        // println!("unrecognized token: {:#?} | {}", token, self.prog_line);
                        tokens.extend(expected.iter().cloned());
                    },
                    ParseError::ExtraToken { token: _} => {
                        // println!("extra token: {:#?} | {}", token, self.prog_line);
                    },
                    ParseError::User { error} => panic!("user error: {} | {}", error, self.prog_line),
                }
            }
            Ok(_expression) => {
                tokens.insert(";".to_string());
                return tokens;
            }
        };

        let mut processed_tokens: HashSet<String> = HashSet::new();
        for token in &tokens {
            let mut processed_line = str::replace(&token, "\"", "");
            if processed_line == "identifier" {
                processed_line = "id".to_string();
            }
            processed_tokens.insert(processed_line);
        }

        if (last_instr == "end_int" || last_instr.starts_with("x") || last_instr.starts_with("y")) && !self.in_set {
            processed_tokens.insert("+".to_string());
            processed_tokens.insert("-".to_string());
            processed_tokens.insert("*".to_string());
        }

        //if there are no bindings
        if self.num_let_bindings == 0 && self.def_bindings.is_empty() {
            processed_tokens.retain(|x| x != "id");
        }
        if self.num_let_bindings == 0 {
            processed_tokens.retain(|x| x != "set");
        }

        if self.in_let_binding {
            processed_tokens.retain(|x| x != "let");
        }

        println!("{:?}", processed_tokens);
        return processed_tokens;
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

    fn update_food(&mut self) {
        let last_instr = self.last_instr.clone();
        let mut rng = thread_rng();
        // let next_instrs = self.generate_instructions(last_instr);
        let next_instrs = self.generate_next_tokens(last_instr);

        let mut token_positions: HashSet<(i32, i32)> = HashSet::new();

        // generate a new token for every instr
        // generate 5 end_int tokens if one token is end_int
        let mut num_new_tokens = next_instrs.len();
        if next_instrs.contains("end_int") {
            num_new_tokens += 4;
        }

        while token_positions.len() < num_new_tokens {
            let mut new_x = rng.gen_range((self.window_start_x + 1)..(self.width - 1));
            let mut new_y = rng.gen_range(1..(self.height - 1));
            while self.snake.is_overlap_except_tail(new_x, new_y) {
                new_x = rng.gen_range((self.window_start_x + 1)..(self.width - 1));
                new_y = rng.gen_range(1..(self.height - 1));
            }
            token_positions.insert((new_x, new_y));
        }

        // add all non end_int food tokens
        self.food_list = vec![];
        for instr in next_instrs {
            if instr != "end_int" {
                let position = token_positions.iter().next().unwrap().clone();
                token_positions.remove(&position);
                let new_food = Food {
                    food_x: position.0,
                    food_y: position.1,
                    instr: instr.to_string(),
                };
                self.food_list.push(new_food);
            }
        }
        // add all end_int food tokens with remaining positions
        // if there were no end_int tokens, this block is skipped
        while token_positions.len() > 0 {
            let position = token_positions.iter().next().unwrap().clone();
            token_positions.remove(&position);
            let new_food = Food {
                food_x: position.0,
                food_y: position.1,
                instr: "end_int".to_string(),
            };
            self.food_list.push(new_food);
        }
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.check_if_the_snake_alive(dir) {
            self.snake.move_forward(dir, self.count_blocks);
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
                let mut instrs = vec![];
                instrs.push(Instr::Push(Val::Reg(Reg::RBP)));
                instrs.push(Instr::IMov(Val::Reg(Reg::RBP), Val::Reg(Reg::RSP)));
                instrs.append(&mut compile_to_instrs(&expression, stack_bindings, &mut variable_types,
                    8, &compilation_bindings));
                instrs.push(Instr::Pop(Val::Reg(Reg::RBP)));
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
        self.last_instr = "".to_string();
        self.prog_line = "( ".to_string();
        self.program.clear();
        self.is_game_over = false;
        self.is_def_line = false;
        self.def_bindings.clear();
        self.reached_goal = false;
        self.num_let_bindings = 0;
        self.in_set = false;
        self.update_food();

    }
}
