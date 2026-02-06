# sndio

[![Build Status](https://github.com/jarmani/sndio-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jarmani/sndio-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/sndio.svg)](https://crates.io/crates/sndio)
[![Documentation](https://docs.rs/sndio/badge.svg)](https://docs.rs/sndio)

Minimal, safe Rust bindings for OpenBSD sndio.

## Features

- Small safe wrapper (`Sndio`, `Mode`, `Par`)
- No `unsafe` required in user code
- Uses sndio defaults by default (`SIO_DEVANY`)

## Example usage

```rust
use sndio::{Mode, Sndio};

fn main() {
    let mut snd = Sndio::open(None, Mode::PLAY, false).expect("sio_open");
    let _ = snd.start();
    let buf = [0u8; 4096];
    let _ = snd.write(&buf);
    let _ = snd.stop();
}
```

## Examples

These examples use dev-only dependencies and are not part of the library:

- Play WAV: `cargo run --example play_wav -- path/to/file.wav`
- Play MP3: `cargo run --example play_mp3 -- path/to/file.mp3`
- Record WAV: `cargo run --example record_wav -- out.wav 5`

## Notes

- The examples are 16-bit PCM only.
- If recording is silent, check `sndioctl` for input mute/level and ensure (not like me) a mic is connected.

