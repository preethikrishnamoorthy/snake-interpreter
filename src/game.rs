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

use super::drawing::{draw_block, draw_rectange};
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

pub struct Game {
    snake: Snake,

    // Food
    food_list: Vec<Food>,
    // Game Space
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
    def_bindings: HashMap<String, i32>

}

impl Game {
    pub fn new(width: i32, height: i32) -> Game {
        Game {
            snake: Snake::new(2, 2),
            waiting_time: 0.0,
            food_list: vec![Food {food_x: 5, food_y: 5, instr: "+".to_string()},
                            Food {food_x: 7, food_y: 3, instr: "-".to_string()},
                            Food {food_x: 2, food_y: 8, instr: "*".to_string()},
                            Food {food_x: 8, food_y: 5, instr: "def".to_string()},
                            Food {food_x: 6, food_y: 6, instr: ";".to_string()},
            ],
            width: width,
            height: height,
            is_game_over: false,
            is_def_line: false,
            prog_line: "( ".to_string(),
            def_bindings: HashMap::new(),
        }
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
        draw_rectange(BORDER_COLOR, 0, 0, self.width, 1, con, g);
        draw_rectange(BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        draw_rectange(BORDER_COLOR, 0, 0, 1, self.height, con, g);
        draw_rectange(BORDER_COLOR, self.width - 1, 0, 1, self.height, con, g);

        // Draw a game-over rectangle
        if self.is_game_over {
            draw_rectange(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g);
        }
    }

    fn instr_to_color(instr: String) -> Color {
        // [red, green, blue, alpha]
        //
        // All values are between 0.0 and 1.0.
        // For example, black is `[0.0, 0.0, 0.0, 1.0]` and white is `[1.0, 1.0, 1.0, 1.0]`.
        if instr == "+" {
            return [0.0, 1.0, 1.0, 1.0]; //yellow
        } else if instr == "-" {
            return [1.0, 1.0, 0.0, 1.0]; //purple
        } else if instr == "*" {
            return [0.0, 0.0, 1.0, 1.0]; //blue
        } else if instr == ";" {
            return [0.0, 1.0, 0.0, 1.0]; //green
        } else if instr == "def" {
            return [1.0, 0.0, 0.0, 1.0]; //red
        } else {
            return FOOD_COLOR;
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
            
            if instr_eaten == ";" {
                self.prog_line.push_str(&next_num_instr);
                self.prog_line.push_str(&" )".to_string());
                println!("{}", self.prog_line);
                
                if self.is_def_line {
                    println!("saving result to var number {}", self.def_bindings.len());
                }

                let res = self.run_line();

                println!("res of running prev line: {}", res);

                self.prog_line = "( ".to_string();
                if self.is_def_line {
                    self.def_bindings.insert("x".to_string() + &self.def_bindings.len().to_string(), res);
                }
                self.is_def_line = false;
            }
            
            else if instr_eaten == "def" {
                self.is_def_line = true;
            }
            
            else {
                println!("ate instr: {}", &instr_eaten.clone());
                self.prog_line.push_str(&next_num_instr);
                self.prog_line.push_str(&" ".to_string());
                self.prog_line.push_str(&instr_eaten.clone());
                self.prog_line.push_str(&" ".to_string());
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
        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    fn update_food(&mut self, last_instr: String) {
        let mut rng = thread_rng();
        let next_instrs = self.generate_instructions(last_instr);

        self.food_list = vec![];
        for instr in next_instrs {
            let mut new_x = rng.gen_range(1..(self.width - 1));
            let mut new_y = rng.gen_range(1..(self.height - 1));
            while self.snake.is_overlap_except_tail(new_x, new_y) {
                new_x = rng.gen_range(1..(self.width - 1));
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

    fn generate_instructions(&mut self, _last_instr: String) -> Vec<String>{
        println!("is_def_line: {}", self.is_def_line);
        if self.is_def_line {
            return vec!["+".to_string(), "-".to_string(), "*".to_string(), ";".to_string()];
        } else {
            return vec!["+".to_string(), "-".to_string(), "*".to_string(), "def".to_string(), ";".to_string()];
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
                println!("{}", error_message);
                return -1;
            }
            Ok(expression) => {
                println!("{:?}", expression);
                let mut ops = dynasmrt::x64::Assembler::new().unwrap();
                let start = ops.offset();
        
                let instrs = compile_to_instrs(&expression, stack_bindings, &mut variable_types,
                    0, &self.def_bindings);
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
        self.snake = Snake::new(2, 2);
        self.waiting_time = 0.0;
        self.update_food("".to_string());
        self.is_game_over = false;
        self.prog_line = "( ".to_string();
        self.is_def_line = false;
        self.def_bindings = HashMap::new();
        // self.finished_prog_line = false;
    }
}
