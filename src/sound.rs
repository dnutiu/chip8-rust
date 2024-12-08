/// SoundModule represents a module which can produce sound.
pub trait SoundModule {
    /// beep makes a beep sound.
    fn beep(&mut self);
}

/// TerminalSound is a simple module that makes terminal beep sound.
pub struct TerminalSound;

impl SoundModule for TerminalSound {
    fn beep(&mut self) {
        print!("\x07");
    }
}
