mod audio;
mod display;

use crate::audio::SquareWave;
use crate::display::SdlDisplay;
use anyhow::anyhow;
use clap::Parser;
use emulator::emulator::Emulator;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;

const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const PIXEL_COLOR: Color = Color::RGB(0, 255, 0);

#[derive(Parser, Debug)]
#[command(
    version = "1.0",
    about = "A Chip8 chip8_core.",
    long_about = "A program which emulates the Chip8 system."
)]
struct CliArgs {
    /// The path to the ROM file to emulate.
    rom_path: String,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();

    let file = File::open(&args.rom_path)?;

    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let mut sdl_display_backend: SdlDisplay = SdlDisplay::new(&sdl_context)?;

    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };
    let audio_device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 25.0,
                volume: 0.25,
            }
        })
        .unwrap();

    let mut emulator = Emulator::new();
    emulator.load_rom(file)?;

    sdl_display_backend.clear();
    loop {
        if emulator.tick() {
            let event = event_pump.poll_event();
            match event {
                Some(Event::Quit { .. }) => {
                    emulator.handle_input(Some(0xFF));
                }
                Some(Event::KeyDown { keycode, .. }) => match keycode {
                    Some(Keycode::ESCAPE) => emulator.handle_input(Some(0xFF)),
                    Some(Keycode::NUM_1) => emulator.handle_input(Some(1)),
                    Some(Keycode::NUM_2) => emulator.handle_input(Some(2)),
                    Some(Keycode::NUM_3) => emulator.handle_input(Some(3)),
                    Some(Keycode::NUM_4) => emulator.handle_input(Some(0xC)),
                    Some(Keycode::Q) => emulator.handle_input(Some(4)),
                    Some(Keycode::W) => emulator.handle_input(Some(5)),
                    Some(Keycode::E) => emulator.handle_input(Some(6)),
                    Some(Keycode::R) => emulator.handle_input(Some(0xD)),
                    Some(Keycode::A) => emulator.handle_input(Some(7)),
                    Some(Keycode::S) => emulator.handle_input(Some(8)),
                    Some(Keycode::D) => emulator.handle_input(Some(9)),
                    Some(Keycode::F) => emulator.handle_input(Some(0xE)),
                    Some(Keycode::Z) => emulator.handle_input(Some(0xA)),
                    Some(Keycode::X) => emulator.handle_input(Some(0)),
                    Some(Keycode::C) => emulator.handle_input(Some(0xB)),
                    Some(Keycode::V) => emulator.handle_input(Some(0xF)),
                    _ => {}
                },
                Some(Event::KeyUp { .. }) => emulator.handle_input(None),
                _ => {}
            }

            if emulator.should_beep() {
                audio_device.resume();
            } else {
                audio_device.pause();
            }

            emulator.execute_and_fetch()?;

            // render
            sdl_display_backend.render(&emulator.get_display_buffer());
        } else {
            sleep(Duration::from_millis(1));
        }
    }
}
