use std::env;
use std::fs::File;
use std::time::{Duration, Instant};

use hound::{SampleFormat, WavSpec, WavWriter};
use sndio::{Mode, Sndio, SIO_LE_NATIVE};

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("usage: record_wav <out.wav> [seconds]");
    let seconds: u64 = args
        .next()
        .as_deref()
        .unwrap_or("5")
        .parse()
        .expect("seconds");

    let mut snd = Sndio::open(None, Mode::REC, false).expect("sio_open");
    let mut par = Sndio::init_par();
    par.rate = 44_100;
    par.rchan = 2;
    par.pchan = 0;
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

    let spec = WavSpec {
        channels: par.rchan as u16,
        sample_rate: par.rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let file = File::create(&path).expect("create wav");
    let mut writer = WavWriter::new(file, spec).expect("wav writer");

    if !snd.start() {
        panic!("sio_start failed");
    }

    let end = Instant::now() + Duration::from_secs(seconds);
    let mut buf = vec![0u8; 4096];
    while Instant::now() < end {
        let n = snd.read(&mut buf);
        if n == 0 {
            break;
        }
        let mut i = 0;
        while i + 1 < n {
            let s = i16::from_le_bytes([buf[i], buf[i + 1]]);
            writer.write_sample(s).expect("write sample");
            i += 2;
        }
    }

    let _ = snd.stop();
    writer.finalize().expect("finalize wav");
}
