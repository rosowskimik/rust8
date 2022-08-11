use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::ArgMatches;
use lazy_static::lazy_static;
use pixels::{Pixels, SurfaceTexture};
use rust8::{
    display::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
    emulator::{ChipConfig, ChipEmulator},
    keypad::ChipKey,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

lazy_static! {
    static ref KEY_MAP: HashMap<ChipKey, VirtualKeyCode> = {
        let mut m = HashMap::with_capacity(16);
        m.insert(ChipKey::Key1, VirtualKeyCode::Key1);
        m.insert(ChipKey::Key2, VirtualKeyCode::Key2);
        m.insert(ChipKey::Key3, VirtualKeyCode::Key3);
        m.insert(ChipKey::KeyC, VirtualKeyCode::Key4);
        m.insert(ChipKey::Key4, VirtualKeyCode::Q);
        m.insert(ChipKey::Key5, VirtualKeyCode::W);
        m.insert(ChipKey::Key6, VirtualKeyCode::E);
        m.insert(ChipKey::KeyD, VirtualKeyCode::R);
        m.insert(ChipKey::Key7, VirtualKeyCode::A);
        m.insert(ChipKey::Key8, VirtualKeyCode::S);
        m.insert(ChipKey::Key9, VirtualKeyCode::D);
        m.insert(ChipKey::KeyE, VirtualKeyCode::F);
        m.insert(ChipKey::KeyA, VirtualKeyCode::Z);
        m.insert(ChipKey::Key0, VirtualKeyCode::X);
        m.insert(ChipKey::KeyB, VirtualKeyCode::C);
        m.insert(ChipKey::KeyF, VirtualKeyCode::V);
        m
    };
}

pub struct Game {
    pub pixels: Pixels,
    pub input: WinitInputHelper,
    pub emulator: ChipEmulator,
    pub rom_loaded: bool,
    pub paused: bool,
}

impl Game {
    pub fn new(pixels: Pixels) -> Self {
        Self {
            pixels,
            input: WinitInputHelper::new(),
            emulator: ChipEmulator::init(),
            rom_loaded: false,
            paused: false,
        }
    }

    pub fn set_emulator_config(&mut self, config: ChipConfig) {
        self.emulator.set_config(config);
    }

    pub fn load_rom(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let reader = BufReader::new(File::open(path)?);
        self.emulator.load_rom(reader)?;
        self.rom_loaded = true;

        Ok(())
    }

    pub fn draw_screen(&mut self) {
        self.pixels
            .get_frame()
            .chunks_exact_mut(4)
            .zip(self.emulator.display())
            .for_each(|(screen_pxl, emulator_pxl)| {
                // screen_pxl.fill(if *emulator_pxl { 0xFF } else { 0x00 });
                if *emulator_pxl {
                    screen_pxl.fill(0xFF);
                } else {
                    screen_pxl[3] /= 2;
                }
            });

        self.pixels.render().expect("Failed to render pixels");
    }

    pub fn handle_event(&mut self, event: &Event<()>) -> bool {
        if self.input.update(event) {
            // Quit requrest
            if self.input.key_pressed(VirtualKeyCode::Escape) || self.input.quit() {
                return true;
            }

            // Pause request
            if self.input.key_pressed(VirtualKeyCode::Space) {
                self.paused = !self.paused;
            }

            // Resize request
            if let Some(size) = self.input.window_resized() {
                self.pixels.resize_surface(size.width, size.height);
            }

            // File drop
            if let Some(path) = self.input.dropped_file() {
                self.emulator.reset();
                self.load_rom(path).expect("Failed to load ROM");
            }

            // Normal controls
            if let Some(chip_key) = self.emulator.current_key() {
                if self.input.key_released(*KEY_MAP.get(chip_key).unwrap()) {
                    // Button released - recheck for key press
                    self.emulator.set_key(None);
                    self.check_keys();
                }
            } else {
                // No button held - check for key press
                self.check_keys();
            }
        }

        false
    }

    fn check_keys(&mut self) {
        for (chip_key, key) in KEY_MAP.iter() {
            if self.input.key_pressed(*key) {
                self.emulator.set_key(Some(*chip_key));
                return;
            }
        }
    }
}

pub fn setup(args: &ArgMatches) -> Result<(EventLoop<()>, Window, Game)> {
    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(DISPLAY_WIDTH as f64, DISPLAY_HEIGHT as f64);
        let scaled_size =
            LogicalSize::new(DISPLAY_WIDTH as f64 * 10.0, DISPLAY_HEIGHT as f64 * 10.0);
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
        Pixels::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, surface_texture)?
    };

    let game = {
        let mut game = Game::new(pixels);

        game.set_emulator_config(ChipConfig {
            modified_shift: *args
                .get_one("modified_shift")
                .expect("Flag should have default value"),
            modified_load: *args
                .get_one("modified_load")
                .expect("Flag should have default value"),
        });

        if let Some(path) = args.get_one::<PathBuf>("rom") {
            game.load_rom(path)?;
        }

        game
    };

    Ok((event_loop, window, game))
}
