# CHIP-8 Assembler

This is a simple assembler for a simple CHIP-8 assembly language, based loosley on [this spec](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) and [this C code](https://github.com/wernsey/chip8/blob/e3e4a5cd81acda3278a394a7598de71ecf7f0c05/c8asm.c). The examples in the `asm/` folder are loosely based on [these examples](https://github.com/wernsey/chip8/tree/e3e4a5cd81acda3278a394a7598de71ecf7f0c05/examples).

## Assembly code

The assembly used by this assembler is slightly different and is described in detail here.

### Notation

For the purposes of this document:

- `Vx`, `Vy` and `Vn` refer to any of the 16 CHIP-8 registers, `V0` through `VF`
- `addr` refers to a 12-bit address in the CHIP-8 interpreter's RAM.
- `nnn` refers to a 12-bit value.
- `n` refers to a 4-bit nibble value.
- `kk` is a byte value.

Decimal literals are a sequence of the characters `0-9`, for example `254`.

Binary literals start with `%` followed by several `0` or `1` symbols,
for example `%11111110`.

Hexadecimal literals start with a `#` followed by characters from
`0-9`, `a-f` or `A-F`, for example `#FE` for 254.

### Summary

| Mnemonic         | Description                                         |
|------------------|-----------------------------------------------------|
|  `nop`           | Do nothing                                          |
|  `cls`           | Clear screen                                        |
|  `ret`           | Return                                              |
|  `jmp addr`      | Jump to `addr`                                      |
|  `jmp v0, addr`  | Jump to `v0 + addr`                                 |
|  `call addr`     | Call routine at `addr`                              |
|  `se Vx, kk`     | Skip if `Vx` equals `kk`                            |
|  `se Vx, Vy`     | Skip if `Vx` equals `Vy`                            |
|  `sne Vx, kk`    | Skip if `Vx` does not equal `kk`                    |
|  `sne Vx, Vy`    | Skip if `Vx` does not equal `Vy`                    |
|  `ld Vx, kk`     | Loads a literal value `kk` into `Vx`                |
|  `ld Vx, Vy`     | Loads register `Vy` into `Vx`                       |
|  `ld I, nnn`     | Loads `nnn` into register `I`                       |
|  `ld Vx, K`      | Loads a key pressed into `Vx`                       |
|  `ld Vx, DT`     | Loads the delay timer into register `Vx`            |
|  `add Vx, kk`    | Add `kk` to register `Vx`                           |
|  `add Vx, Vy`    | Add the value in `Vy` to register `Vx`              |
|  `add I, Vx`     | Add the value in `Vx` to register `I`               |
|  `or Vx, Vy`     | Bitwise OR the value in `Vy` with register `Vx`     |
|  `and Vx, Vy`    | Bitwise AND the value in `Vy` with register `Vx`    |
|  `xor Vx, Vy`    | Bitwise XOR the value in `Vy` with register `Vx`    |
|  `sub Vx, Vy`    | Subtract the value in `Vy` from `Vx`                |
|  `subn Vx, Vy`   | Subtract the value in `Vy` from `Vx`                |
|  `shr Vx`        | Shift `Vx` to the right by 1 place                  |
|  `shl Vx`        | Shift `Vx` to the left by 1 place                   |
|  `rnd Vx, kk`    | Random number AND `kk` into `Vx`                    |
|  `drw Vx, Vy, n` | Draw a sprite of `n` rows at `Vx, Vy`               |
|  `skp Vx`        | Skip if key in `Vx` pressed                         |
|  `sknp Vx`       | Skip if key in `Vx` not pressed                     |
|  `delay Vx`      | Loads register `Vx` into the delay timer            |
|  `sound Vx`      | Loads register `Vx` into the sound timer            |
|  `font Vx`       | Loads the 8x5 font sprite of `Vx` into `I`          |
|  `bcd Vx`        | Load BCD value of `Vx` into `I` to `I+2`            |
|  `stor Vx`       | Stores `V0` through `Vx` to the address in `I`      |
|  `rstr Vx`       | Restores `V0` through `Vx` from the address in `I`  |
