use std::env;
use std::fs::File;
use std::io::BufReader;

use hound::WavReader;
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

fn main() {
    let path = env::args().nth(1).expect("usage: play_wav <file.wav>");
    let file = File::open(&path).expect("open wav");
    let reader = BufReader::new(file);
    let mut wav = WavReader::new(reader).expect("read wav");
    let spec = wav.spec();

    if spec.bits_per_sample != 16 || spec.sample_format != hound::SampleFormat::Int {
        panic!("only 16-bit PCM wav is supported");
    }

    let mut snd = Sndio::open(None, Mode::PLAY, false).expect("sio_open");
    let mut par = Sndio::init_par();
    par.rate = spec.sample_rate;
    par.pchan = spec.channels as u32;
    par.rchan = 0;
    par.bits = 16;
    par.bps = 2;
    par.sig = 1;
    par.le = SIO_LE_NATIVE;
    par.msb = 1;
    if !snd.set_par(&mut par) {
        panic!("sio_setpar failed");
    }
    if !snd.get_par(&mut par) {
        panic!("sio_getpar failed");
    }
    if par.bits != 16 || par.bps != 2 || par.sig != 1 || par.le != 1 {
        panic!("device does not support 16-bit little-endian PCM");
    }

    if !snd.start() {
        panic!("sio_start failed");
    }

    let mut buf = Vec::with_capacity(4096);
    for sample in wav.samples::<i16>() {
        let s = sample.expect("read sample");
        buf.extend_from_slice(&s.to_le_bytes());
        if buf.len() >= 4096 {
            if !write_all(&mut snd, &buf) {
                break;
            }
            buf.clear();
        }
    }
    if !buf.is_empty() {
        let _ = write_all(&mut snd, &buf);
    }
    let _ = snd.stop();
}
