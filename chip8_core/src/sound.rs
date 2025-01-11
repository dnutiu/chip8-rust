/// SoundModule represents a module which can produce sound.
pub trait SoundModule {
    /// beep makes a beep sound.
    fn beep(&mut self);
}
