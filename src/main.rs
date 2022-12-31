use std::{env::args, fs};

use emulator::Emulator;
use sfml::{
    graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable},
    system::Vector2f,
    window::{Event, Style},
};

mod emulator;

const PIXEL_SIZE: u32 = 20;

fn main() -> std::io::Result<()> {
    let rom_path = args()
        .into_iter()
        .collect::<Vec<_>>()
        .get(1)
        .cloned()
        .expect("File name required");

    let program = fs::read(rom_path)?;
    let mut emulator = Emulator::new();
    emulator.load_program(&program);

    let mut window = RenderWindow::new(
        (64 * PIXEL_SIZE, 32 * PIXEL_SIZE),
        "Chip8",
        Style::CLOSE,
        &Default::default(),
    );
    window.set_framerate_limit(60);

    window.set_active(true);
    window.clear(Color::BLACK);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }

        let op = emulator.cycle();
        println!("{:#x}", op);
        if emulator.should_draw() {
            let screen = emulator.get_screen();
            for (i, y) in screen.iter().enumerate() {
                for (j, x) in y.iter().enumerate() {
                    let mut rect = RectangleShape::new();
                    rect.set_size(Vector2f::new(PIXEL_SIZE as f32, PIXEL_SIZE as f32));
                    rect.set_position(Vector2f::new(
                        (j as f32) * (PIXEL_SIZE as f32),
                        (i as f32) * (PIXEL_SIZE as f32),
                    ));

                    if *x == 1 {
                        rect.set_fill_color(Color::GREEN);
                    } else {
                        rect.set_fill_color(Color::BLACK);
                    }
                    window.draw(&rect);
                }
            }
        }

        window.display();
    }

    Ok(())
}
