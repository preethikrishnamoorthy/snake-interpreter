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
        text::Text::new_color([1.0, 1.0, 1.0, 1.0], 15) // Text color and font size
            .draw(
                op, // The text to display
                font,
                &new_draw_state,
                con.transform.trans(gui_x + BLOCK_SIZE / 3.0, gui_y + BLOCK_SIZE / 2.0),
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

// used to draw heap and temp vars
pub fn draw_text(text: String, color: Color, start_x: f64, start_y: f64, con: &Context, g: &mut G2d, font: &mut Glyphs) {
    // let gui_x = to_gui_coord(start_x);
    // let gui_y = to_gui_coord(start_y);
    let new_draw_state = con.draw_state.clone();

    text::Text::new_color(color, 18) // Text color and font size
        .draw(
            &text.to_string(),
            font,
            &new_draw_state,
            con.transform.trans(start_x, start_y),
            g,
        )
        .unwrap_or_else(|e| {
            eprintln!("Error drawing text: {:?}", e);  // Print error if text drawing fails
        });
}

pub fn draw_blocks_count(blocks_num: i64, con: &Context, g: &mut G2d, font: &mut Glyphs) {
    // Get the width of the Block Count string
    let width: f64 = "Block Count: "
        .chars()
        .map(|ch| font.character(20, ch).unwrap().advance_width())
        .sum();

    let gui_x = width + to_gui_coord(1);
    let gui_y = 30.0;
    let new_draw_state = con.draw_state.clone();

    text::Text::new_color([1.0, 1.0, 1.0, 1.0], 15) // Text color and font size
        .draw(
            &blocks_num.to_string(),
            font,
            &new_draw_state,
            con.transform.trans(gui_x, gui_y),
            g,
        )
        .unwrap_or_else(|e| {
            eprintln!("Error drawing text: {:?}", e);  // Print error if text drawing fails
        });
}

pub fn draw_program_line(program_line: String, result: Option<i32>, x: i32, y: i32, con: &Context, g: &mut G2d, font: &mut Glyphs) {
    let gui_x = to_gui_coord(x);
    let gui_y = to_gui_coord(y);

    let new_draw_state = con.draw_state.clone();

    let font_size = 20;

    // draw program line
    text::Text::new_color([1.0, 1.0, 1.0, 1.0], font_size) // Text color and font size
        .draw(
            &program_line, // The text to display
            font,
            &new_draw_state,
            con.transform.trans(gui_x, gui_y),
            g,
        )
        .unwrap_or_else(|e| {
            eprintln!("Error drawing text: {:?}", e);  // Print error if text drawing fails
        });

    // Get the width of the program line string
    let width: f64 = program_line
        .chars()
        .map(|ch| font.character(font_size, ch).unwrap().advance_width())
        .sum();

    // if it exists, draw the result in blue
    match result {
        Some(res) => {
            let mut text_to_draw = "".to_string();
            text_to_draw.push_str(" -> ");
            text_to_draw.push_str(&res.to_string());
            text::Text::new_color([0.0, 0.0, 1.0, 1.0], font_size) // Text color and font size
            .draw(
                &text_to_draw, // The text to display
                font,
                &new_draw_state,
                con.transform.trans(gui_x + width, gui_y),
                g,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error drawing text: {:?}", e);  // Print error if text drawing fails
            });
        },
        None => ()
    }

}
