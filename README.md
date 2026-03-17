# chip8

A CHIP-8 emulator written in Rust. Nothing groundbreaking, just a fun weekend project to learn more about emulation and Rust.

![screenshot](./.assets/screenshot.png)

## What is CHIP-8?

CHIP-8 is an interpreted language from the 1970s, originally designed for the COSMAC VIP microcomputer. It's basically the "hello world" of emulation projects — simple enough to finish, but still teaches you the core concepts.

## Building

```bash
git clone https://github.com/Plastikpizza/chip8
cd chip8
cargo build --release
```

## Usage

```bash
cargo run --release -- path/to/rom.ch8
```

A few ROMs to try if you don't have any: [chip8Archive](https://github.com/JohnEarnest/chip8Archive)

## Controls

```
CHIP-8 keypad → keyboard
1 2 3 C       → 1 2 3 4
4 5 6 D       → Q W E R
7 8 9 E       → A S D F
A 0 B F       → Z X C V
```

## Dependencies

- `minifb` — windowing and input

## Known issues

- Sound is not implemented (the beep timer does nothing)
- Some ROMs behave unexpectedly due to quirks differences between interpreters

## References

- [Cowgod's CHIP-8 reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) — basically required reading
- [Tobias V. Langhoff's quirks guide](https://chip8.gulrak.net/) — helpful once things mostly work but some ROMs still don't
