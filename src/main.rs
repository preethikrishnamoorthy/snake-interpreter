extern crate piston_window;
extern crate rand;

use piston_window::*;
use piston_window::types::Color;

use snake_interpreter::game::Game;
use snake_interpreter::drawing::to_gui_coord_u32;

const BACK_COLOR: Color = [0.204, 0.286, 0.369, 1.0];

fn main() {
    let (width, height) = (40, 20);
    let game_window_width = (to_gui_coord_u32(width) as f64) / 2.0;

    // Prepare window settings
    let mut window_settings = WindowSettings::new("Snake to Snek",
    [to_gui_coord_u32(width), to_gui_coord_u32(height)]).exit_on_esc(true);

    // Fix vsync extension error for linux
    window_settings.set_vsync(true); 

    // Create a window
    let mut window: PistonWindow = window_settings.build().unwrap();

    // Create a snake
    let mut game = Game::new(width / 2, height);
    let mut game_started = false;
    let mut font = window.load_font("src/Poppins-Bold.ttf").unwrap(); // Load a font

    // Event loop
    while let Some(event) = window.next() {

        if game_started {
            // Catch the events of the keyboard
            if let Some(Button::Keyboard(key)) = event.press_args() {
                game.key_pressed(key);
            }

            // Draw all of them
            window.draw_2d(&event, |c, g, device| {
                clear(BACK_COLOR, g);
                line(
                    [0.0, 0.0, 1.0, 1.0],
                    2.0, // line thickness
                    [game_window_width, 0.0, game_window_width, to_gui_coord_u32(height) as f64],
                    c.transform,
                    g,
                );
                let transform = c.transform.trans(700.0, 30.0); // Position for the text
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 30)
                    .draw(
                        "Code",
                        &mut font,
                        &DrawState::default(),
                        transform,
                        g,
                    )
                    .unwrap();
                font.factory.encoder.flush(device);
                game.draw(&c, g, &mut font);
                font.factory.encoder.flush(device);
            });

            // Update the state of the game
            event.update(|arg| {
                game.update(arg.dt);
            });

            // if game.check_line_generated() {
            //     run_line(game.get_prog());
            // }
        } else {
            if let Some(Button::Keyboard(Key::S)) = event.press_args() {
                game_started = true; // Start the game when "S" is pressed
            }
            window.draw_2d(&event, |c, g, device| {
                clear(BACK_COLOR, g); // Gray background for the start screen

                let transform = c.transform.trans(400.0, 240.0); // Position for the text
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32)
                    .draw(
                        "Press 'S' to Start",
                        &mut font,
                        &DrawState::default(),
                        transform,
                        g,
                    )
                    .unwrap();
                font.factory.encoder.flush(device);
            });
        }

        
    }
}
