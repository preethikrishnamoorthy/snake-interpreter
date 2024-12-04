extern crate piston_window;
extern crate rand;

use piston_window::*;
use piston_window::types::Color;

use snake_interpreter::game::Game;
use snake_interpreter::drawing::to_gui_coord_u32;

const BACK_COLOR: Color = [0.204, 0.286, 0.369, 1.0];

fn main() {
    let (width, height) = (20, 20);

    // Prepare window settings
    let mut window_settings = WindowSettings::new("Rust Snake",
    [to_gui_coord_u32(width), to_gui_coord_u32(height)]).exit_on_esc(true);

    // Fix vsync extension error for linux
    window_settings.set_vsync(true); 

    // Create a window
    let mut window: PistonWindow = window_settings.build().unwrap();

    // Create a snake
    let mut game = Game::new(width, height);

    // Event loop
    while let Some(event) = window.next() {

        // Catch the events of the keyboard
        if let Some(Button::Keyboard(key)) = event.press_args() {
            game.key_pressed(key);
        }
        
        let mut font = window.load_font("src/Poppins-Bold.ttf").unwrap(); // Load a font

        // Draw all of them
        window.draw_2d(&event, |c, g, _| {
            clear(BACK_COLOR, g);
            game.draw(&c, g, &mut font);
        });

        // Update the state of the game
        event.update(|arg| {
            game.update(arg.dt);
        });

        // if game.check_line_generated() {
        //     run_line(game.get_prog());
        // }
    }
}
