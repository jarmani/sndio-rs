use std::env;
use std::fs::File;

use minimp3::{Decoder, Frame};
use sndio::{Mode, Sndio, SIO_LE_NATIVE};

fn write_all(snd: &mut Sndio, mut data: &[u8]) -> bool {
    while !data.is_empty() {
        let written = snd.write(data);
        if written == 0 {
            return false;
        }
        data = &data[written..];
    }
    true
}

fn frame_to_bytes(frame: &Frame) -> Vec<u8> {
    let mut buf = Vec::with_capacity(frame.data.len() * 2);
    for s in &frame.data {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf
}

fn main() {
    let path = env::args().nth(1).expect("usage: play_mp3 <file.mp3>");
    let file = File::open(&path).expect("open mp3");
    let mut dec = Decoder::new(file);

    let mut snd: Option<Sndio> = None;
    let mut expected_rate = 0;
    let mut expected_channels = 0;

    while let Ok(frame) = dec.next_frame() {
        if snd.is_none() {
            let mut s = Sndio::open(None, Mode::PLAY, false).expect("sio_open");
            let mut par = Sndio::init_par();
            par.rate = frame.sample_rate as u32;
            par.pchan = frame.channels as u32;
            par.rchan = 0;
            par.bits = 16;
            par.bps = 2;
            par.sig = 1;
            par.le = SIO_LE_NATIVE;
            par.msb = 1;
            if !s.set_par(&mut par) {
                panic!("sio_setpar failed");
            }
            if !s.get_par(&mut par) {
                panic!("sio_getpar failed");
            }
            if par.bits != 16 || par.bps != 2 || par.sig != 1 || par.le != 1 {
                panic!("device does not support 16-bit little-endian PCM");
            }
            if !s.start() {
                panic!("sio_start failed");
            }
            expected_rate = frame.sample_rate;
            expected_channels = frame.channels;
            snd = Some(s);
        }

        if frame.sample_rate != expected_rate || frame.channels != expected_channels {
            panic!("unexpected mp3 stream change");
        }

        let buf = frame_to_bytes(&frame);
        if let Some(ref mut s) = snd {
            if !write_all(s, &buf) {
                break;
            }
        }
    }
    if let Some(mut s) = snd {
        let _ = s.stop();
    }
}
