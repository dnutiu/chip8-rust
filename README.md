# Chip8 Emulator

![logo](./chip8-project.png)

A Chip8 emulator written in [Rust](https://www.rust-lang.org/) (1.83.0) for learning and educational purposes.

In turns inside your Terminal and under SDL2 and the code can be easily modified to run in other envionments.

See it in action:

### Breakout (Sdl2)

https://videos.nuculabs.dev/w/pD1U8aXToVmNhuZdrCe8qV

### Space Invaders (Terminal Old)

[![asciicast](https://asciinema.org/a/694431.svg)](https://asciinema.org/a/694431)

### Pong (Terminal Old)

[![asciicast](https://asciinema.org/a/Q7TCN6e1V2y2Vfm2tCiCSzsAd.svg)](https://asciinema.org/a/Q7TCN6e1V2y2Vfm2tCiCSzsAd)

# Project organization

The project is organized under three crates.

The `chip8_core` crate contains the core functionality for the emulator and can be compiled without
depending on Rust's std (standard) library, this makes it possible to run it on embedded systems.

The `chip8_sdl` crate is an emulator implementation that uses sdl for graphics and audio.

The `chip8_tui` crate is an emulator implementation that runs inside the terminal.

## Tests

You can run tests with `cargo test`

---

## About Chip8

CHIP-8 is an interpreted programming language and a simple virtual machine designed primarily for running on 8-bit microcomputers and other systems in the late 1970s and early 1980s. Created by Joseph Weisbecker for RCA's COSMAC VIP microcomputer, CHIP-8 was intended to make programming more accessible by providing a higher-level language that could abstract away the complexities of direct hardware manipulation.

The CHIP-8 architecture features a straightforward design:

    CPU: It uses a 16-bit address space and has 16 8-bit registers (V0-VF), where VF often serves as a flag register. Instructions are 2 bytes long, allowing for 35 possible opcodes.
    Memory: There are 4KB (4096 bytes) of RAM, with the first 512 bytes reserved for the interpreter itself, leaving 3584 bytes for programs and data.
    Display: A 64x32 pixel monochrome display where each pixel can be either on or off.
    Input: It handles input through a 16-key hexadecimal keypad.
    Timers: Two timers (delay and sound) decrement at 60 Hz, providing simple timing mechanisms.


CHIP-8 games and programs were typically small, often fitting within the limited memory constraints, and included classics like "Pong", "Space Invaders", and "Tetris" adaptations. The simplicity of CHIP-8 has made it a popular choice for educational purposes, teaching fundamentals of computer architecture, programming, and emulation. Modern emulators and interpreters for CHIP-8 exist on various platforms, including web browsers, allowing enthusiasts to run and even develop new games for this vintage system. This has kept CHIP-8 relevant as a tool for understanding basic computing concepts and low-level programming.

## How is this project organized

The project is written in Rust and it's organized in the following modules:

```shell
.
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── proprietary_roms # Roms which I have no permission to share here.
├── README.md
├── roms
│   ├── 1-chip8-logo.ch8 # Chip8 Logo Test ROM
│   ├── 3-corax+.ch8 # Corax+ Instructions Test ROM
│   └── ibm-logo.ch8 # IBM Logo Test ROM
├── src
│   ├── display.rs # The screen / display module.
│   ├── chip8_core.rs # The chip8_core logic which emulates the CPU.
│   ├── input.rs # The input logic.
│   ├── instruction.rs # The instruction decoding logic.
│   ├── main.rs # The main file. This is the entrypoint.
│   ├── sound.rs # The sound module.
│   └── stack.rs # A stack implementation.
```

## Resources

I've used the following resources to implement the project:

- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
- https://github.com/Timendus/chip8-test-suite

and got inspired by other people's code when I was confused by an instruction. 


---
Made with ❤️ by [NucuLabs.dev](https://blog.nuculabs.dev)

Follow me on 🦋 or 🐘 and let's chat: 
- BlueSky: [@nuculabs.dev](https://bsky.app/profile/nuculabs.dev)
- Mastodon: [@nuculabs](https://mastodon.social/@nuculabs)