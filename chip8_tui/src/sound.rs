use emulator::sound::SoundModule;

/// TerminalSound is a simple module that makes terminal beep sound.
pub struct TerminalSound;

impl SoundModule for TerminalSound {
    fn beep(&mut self) {
        print!("\x07");
    }
}
