use piston_window::types::Color;
use piston_window::*;

const BLOCK_SIZE: f64 = 25.0;

pub fn to_gui_coord(game_coord: i32) -> f64 {
    (game_coord as f64) * BLOCK_SIZE
}

pub fn to_gui_coord_u32(game_coord: i32) -> u32 {
    to_gui_coord(game_coord) as u32
}

pub fn draw_block(color: Color, op: &str, x: i32, y: i32, con: &Context, g: &mut G2d, font: &mut Glyphs) {
    let gui_x = to_gui_coord(x);
    let gui_y = to_gui_coord(y);
    let new_draw_state = con.draw_state.clone();
    rectangle(color, [gui_x, gui_y,
        BLOCK_SIZE, BLOCK_SIZE], con.transform, g);

    if op != "snake" {
        text::Text::new_color([1.0, 1.0, 1.0, 1.0], 10) // Text color and font size
            .draw(
                op, // The text to display
                font,
                &new_draw_state,
                con.transform.trans(gui_x + BLOCK_SIZE / 2.0, gui_y + BLOCK_SIZE / 2.0),
                g,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error drawing text: {:?}", e);  // Print error if text drawing fails
            });
    }
}


pub fn draw_rectange(color: Color, start_x: i32, start_y: i32, width: i32, height: i32, con: &Context, g: &mut G2d) {
    let gui_start_x = to_gui_coord(start_x);
    let gui_start_y = to_gui_coord(start_y);

    rectangle(color, [gui_start_x, gui_start_y,
            BLOCK_SIZE * (width as f64), BLOCK_SIZE * (height as f64)], con.transform, g);
}
