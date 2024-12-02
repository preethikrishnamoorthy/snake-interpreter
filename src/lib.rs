pub mod utils;
pub mod lexer;
pub mod tokens;
pub mod snake;
pub mod game;
pub mod drawing;
pub mod compile;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);
