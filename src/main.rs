use emulator::Emulator;
use sfml::{
    graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable},
    system::Vector2f,
    window::{Event, Key, Style},
};
use std::{env::args, fs};

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
    window.set_framerate_limit(120);

    window.set_active(true);
    window.clear(Color::BLACK);

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed {
                    code,
                    alt: _,
                    ctrl: _,
                    shift: _,
                    system: _,
                } => {
                    let index = process_key(code);
                    match index {
                        Some(val) => emulator.set_key_state(val, true),
                        None => {}
                    }
                }
                Event::KeyReleased {
                    code: _,
                    alt: _,
                    ctrl: _,
                    shift: _,
                    system: _,
                } => {
                    for i in 0..16 {
                        emulator.set_key_state(i, false);
                    }
                }
                _ => {}
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

fn process_key(key: Key) -> Option<usize> {
    match key {
        Key::Num1 => Some(0),
        Key::Num2 => Some(1),
        Key::Num3 => Some(2),
        Key::Num4 => Some(3),

        Key::A => Some(4),
        Key::Z => Some(5),
        Key::E => Some(6),
        Key::R => Some(7),

        Key::Q => Some(8),
        Key::S => Some(9),
        Key::D => Some(10),
        Key::F => Some(11),

        Key::W => Some(12),
        Key::X => Some(13),
        Key::C => Some(14),
        Key::V => Some(15),

        _ => None,
    }
}
