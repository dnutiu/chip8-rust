/// SoundModule represents a module which can produce sound.
pub trait SoundModule {
    /// beep makes a beep sound.
    fn beep(&mut self);
}

/// A simple module for testing the sound.
pub(crate) struct TestingSound;

impl SoundModule for TestingSound {
    fn beep(&mut self) {}
}
