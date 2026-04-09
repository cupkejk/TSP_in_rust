use macroquad::prelude::*;

#[macroquad::main("Travelling Salesman Problem")]
async fn main() {
    // Game loop
    loop {
        // 1. Logic & Input
        if is_key_down(KeyCode::Escape) {
            break;
        }

        // 2. Rendering
        clear_background(LIGHTGRAY);

        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);
        
        draw_circle(
            screen_width() / 2.0,
            screen_height() / 2.0,
            50.0,
            RED,
        );

        // 3. Wait for the next frame
        next_frame().await
    }
}