//! This module handles memory.

/// Initialize a new 4k block of memory with the given rom loaded in at address 0x200.
pub fn init_memory(rom: &[u8]) -> [u8; 4096] {
    let mut mem = [0; _];

    for (offset, &byte) in rom.iter().enumerate() {
        mem[0x200 + offset] = byte;
    }

    // TODO: Populate the font in the interpreter section of memory

    mem
}
