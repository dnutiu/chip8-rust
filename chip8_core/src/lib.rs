#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod display;
pub mod emulator;
pub mod instruction;
pub mod read;
pub mod stack;