#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chip8::{SCREEN_HEIGHT, SCREEN_WIDTH};

use game_loop::game_loop;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize, event::VirtualKeyCode, event_loop::EventLoop, window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

struct Game {
    pixels: Pixels,
    input: WinitInputHelper,
    pause: bool,
    counter: u32,
}

impl Game {
    fn new(pixels: Pixels) -> Self {
        Self {
            pixels,
            input: WinitInputHelper::new(),
            pause: false,
            counter: 0,
        }
    }

    fn update_controls(&mut self) {
        if self.input.key_pressed(VirtualKeyCode::Space) {
            self.pause = !self.pause;
        }

        if self.input.key_pressed(VirtualKeyCode::R) {
            self.counter = 0;
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        let scaled_size = LogicalSize::new(SCREEN_WIDTH as f64 * 10.0, SCREEN_HEIGHT as f64 * 10.0);
        WindowBuilder::new()
            .with_title("Rust8")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?
    };

    let game = Game::new(pixels);

    game_loop(
        event_loop,
        window,
        game,
        24,
        0.1,
        move |g| {
            if !g.game.pause {
                g.game.counter += 1;
            }
        },
        move |g| {
            for (i, pixel) in g.game.pixels.get_frame().chunks_exact_mut(4).enumerate() {
                if i == g.game.counter as usize {
                    pixel[0] = 0xFF;
                    pixel[3] = 0xFF;
                } else {
                    pixel.fill(0);
                }
            }

            g.game.pixels.render().expect("Lol nope");
        },
        move |g, event| {
            if g.game.input.update(event) {
                g.game.update_controls();

                if g.game.input.key_pressed(VirtualKeyCode::Escape) || g.game.input.quit() {
                    g.exit();
                }

                if let Some(size) = g.game.input.window_resized() {
                    g.game.pixels.resize_surface(size.width, size.height);
                }
            }
        },
    );
}
