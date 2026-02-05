use std::ffi::{CStr, CString, NulError};

mod raw {
    pub use libc::{c_char, c_int, c_uint, c_void, size_t};

    #[repr(C)]
    pub struct sio_hdl {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct pollfd {
        pub fd: c_int,
        pub events: c_int,
        pub revents: c_int,
    }

    #[repr(C)]
    pub struct sio_par {
        pub bits: c_uint,
        pub bps: c_uint,
        pub sig: c_uint,
        pub le: c_uint,
        pub msb: c_uint,
        pub rchan: c_uint,
        pub pchan: c_uint,
        pub rate: c_uint,
        pub bufsz: c_uint,
        pub xrun: c_uint,
        pub round: c_uint,
        pub appbufsz: c_uint,
        pub __pad: [c_int; 3],
        pub __magic: c_uint,
    }

    pub const SIO_PLAY: c_uint = 1;
    pub const SIO_REC: c_uint = 2;

    pub const SIO_IGNORE: c_uint = 0;
    pub const SIO_SYNC: c_uint = 1;
    pub const SIO_ERROR: c_uint = 2;

    // C macro: #define SIO_DEVANY "default"
    pub const SIO_DEVANY: &[u8] = b"default\0";

    #[cfg(target_endian = "little")]
    pub const SIO_LE_NATIVE: c_uint = 1;
    #[cfg(target_endian = "big")]
    pub const SIO_LE_NATIVE: c_uint = 0;

    pub const SIO_MAXVOL: c_uint = 127;

    pub const fn sio_bps(bits: c_uint) -> c_uint {
        if bits <= 8 {
            1
        } else if bits <= 16 {
            2
        } else {
            4
        }
    }

    #[link(name = "sndio")]
    extern "C" {
        pub fn sio_initpar(par: *mut sio_par);
        pub fn sio_open(dev: *const c_char, mode: c_uint, nbio: c_int) -> *mut sio_hdl;
        pub fn sio_close(hdl: *mut sio_hdl);
        pub fn sio_setpar(hdl: *mut sio_hdl, par: *mut sio_par) -> c_int;
        pub fn sio_getpar(hdl: *mut sio_hdl, par: *mut sio_par) -> c_int;
        pub fn sio_start(hdl: *mut sio_hdl) -> c_int;
        pub fn sio_stop(hdl: *mut sio_hdl) -> c_int;
        pub fn sio_flush(hdl: *mut sio_hdl) -> c_int;
        pub fn sio_read(hdl: *mut sio_hdl, buf: *mut c_void, nbytes: size_t) -> size_t;
        pub fn sio_write(hdl: *mut sio_hdl, buf: *const c_void, nbytes: size_t) -> size_t;
        pub fn sio_nfds(hdl: *mut sio_hdl) -> c_int;
        pub fn sio_pollfd(hdl: *mut sio_hdl, pfd: *mut pollfd, events: c_int) -> c_int;
        pub fn sio_revents(hdl: *mut sio_hdl, pfd: *mut pollfd) -> c_int;
        pub fn sio_eof(hdl: *mut sio_hdl) -> c_int;
        pub fn sio_setvol(hdl: *mut sio_hdl, vol: c_uint) -> c_int;
    }
}


pub const SIO_PLAY: u32 = raw::SIO_PLAY;
pub const SIO_REC: u32 = raw::SIO_REC;

pub const SIO_IGNORE: u32 = raw::SIO_IGNORE;
pub const SIO_SYNC: u32 = raw::SIO_SYNC;
pub const SIO_ERROR: u32 = raw::SIO_ERROR;

pub const SIO_DEVANY: &[u8] = raw::SIO_DEVANY;

pub const SIO_LE_NATIVE: u32 = raw::SIO_LE_NATIVE;
pub const SIO_MAXVOL: u32 = raw::SIO_MAXVOL;

pub const fn sio_bps(bits: u32) -> u32 {
    raw::sio_bps(bits)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Mode(pub u32);

impl Mode {
    pub const PLAY: Mode = Mode(raw::SIO_PLAY);
    pub const REC: Mode = Mode(raw::SIO_REC);
}

impl std::ops::BitOr for Mode {
    type Output = Mode;

    fn bitor(self, rhs: Mode) -> Mode {
        Mode(self.0 | rhs.0)
    }
}

pub struct Sndio {
    hdl: *mut raw::sio_hdl,
}

unsafe impl Send for Sndio {}
unsafe impl Sync for Sndio {}

impl Sndio {
    pub fn open(dev: Option<&CStr>, mode: Mode, nbio: bool) -> Option<Self> {
        let dev_ptr = dev
            .map(|d| d.as_ptr())
            .unwrap_or_else(|| raw::SIO_DEVANY.as_ptr().cast());
        let hdl = unsafe { raw::sio_open(dev_ptr, mode.0 as raw::c_uint, nbio as raw::c_int) };
        if hdl.is_null() {
            None
        } else {
            Some(Self { hdl })
        }
    }

    pub fn open_str(dev: Option<&str>, mode: Mode, nbio: bool) -> Result<Option<Self>, NulError> {
        let dev_c = match dev {
            Some(d) => Some(CString::new(d)?),
            None => None,
        };
        Ok(Self::open(dev_c.as_deref(), mode, nbio))
    }

    pub fn init_par() -> Par {
        let mut par = std::mem::MaybeUninit::<raw::sio_par>::uninit();
        unsafe {
            raw::sio_initpar(par.as_mut_ptr());
            Par::from_raw(par.assume_init())
        }
    }

    pub fn set_par(&mut self, par: &mut Par) -> bool {
        let mut raw_par = par.to_raw();
        let ok = unsafe { raw::sio_setpar(self.hdl, &mut raw_par) == 1 };
        *par = Par::from_raw(raw_par);
        ok
    }

    pub fn get_par(&mut self, par: &mut Par) -> bool {
        let mut raw_par = par.to_raw();
        let ok = unsafe { raw::sio_getpar(self.hdl, &mut raw_par) == 1 };
        *par = Par::from_raw(raw_par);
        ok
    }

    pub fn start(&mut self) -> bool {
        unsafe { raw::sio_start(self.hdl) == 1 }
    }

    pub fn stop(&mut self) -> bool {
        unsafe { raw::sio_stop(self.hdl) == 1 }
    }

    pub fn flush(&mut self) -> bool {
        unsafe { raw::sio_flush(self.hdl) == 1 }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        unsafe { raw::sio_read(self.hdl, buf.as_mut_ptr().cast(), buf.len()) }
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        unsafe { raw::sio_write(self.hdl, buf.as_ptr().cast(), buf.len()) }
    }

    pub fn nfds(&mut self) -> i32 {
        unsafe { raw::sio_nfds(self.hdl) }
    }

    pub fn pollfd(&mut self, pfd: &mut PollFd, events: i32) -> i32 {
        let mut raw_pfd = pfd.to_raw();
        let res = unsafe { raw::sio_pollfd(self.hdl, &mut raw_pfd, events as raw::c_int) };
        pfd.update_from_raw(raw_pfd);
        res
    }

    pub fn revents(&mut self, pfd: &mut PollFd) -> i32 {
        let mut raw_pfd = pfd.to_raw();
        let res = unsafe { raw::sio_revents(self.hdl, &mut raw_pfd) };
        pfd.update_from_raw(raw_pfd);
        res
    }

    pub fn eof(&mut self) -> bool {
        unsafe { raw::sio_eof(self.hdl) == 1 }
    }

    pub fn set_volume(&mut self, vol: u32) -> bool {
        unsafe { raw::sio_setvol(self.hdl, vol as raw::c_uint) == 1 }
    }
}

impl Drop for Sndio {
    fn drop(&mut self) {
        if !self.hdl.is_null() {
            unsafe { raw::sio_close(self.hdl) };
            self.hdl = std::ptr::null_mut();
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PollFd {
    pub fd: i32,
    pub events: i32,
    pub revents: i32,
}

impl PollFd {
    fn to_raw(&self) -> raw::pollfd {
        raw::pollfd {
            fd: self.fd as raw::c_int,
            events: self.events as raw::c_int,
            revents: self.revents as raw::c_int,
        }
    }

    fn update_from_raw(&mut self, raw_pfd: raw::pollfd) {
        self.fd = raw_pfd.fd;
        self.events = raw_pfd.events;
        self.revents = raw_pfd.revents;
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Par {
    pub bits: u32,
    pub bps: u32,
    pub sig: u32,
    pub le: u32,
    pub msb: u32,
    pub rchan: u32,
    pub pchan: u32,
    pub rate: u32,
    pub bufsz: u32,
    pub xrun: u32,
    pub round: u32,
    pub appbufsz: u32,
    _pad: [i32; 3],
    _magic: u32,
}

impl Par {
    fn from_raw(raw_par: raw::sio_par) -> Par {
        Par {
            bits: raw_par.bits,
            bps: raw_par.bps,
            sig: raw_par.sig,
            le: raw_par.le,
            msb: raw_par.msb,
            rchan: raw_par.rchan,
            pchan: raw_par.pchan,
            rate: raw_par.rate,
            bufsz: raw_par.bufsz,
            xrun: raw_par.xrun,
            round: raw_par.round,
            appbufsz: raw_par.appbufsz,
            _pad: raw_par.__pad,
            _magic: raw_par.__magic,
        }
    }

    fn to_raw(&self) -> raw::sio_par {
        raw::sio_par {
            bits: self.bits,
            bps: self.bps,
            sig: self.sig,
            le: self.le,
            msb: self.msb,
            rchan: self.rchan,
            pchan: self.pchan,
            rate: self.rate,
            bufsz: self.bufsz,
            xrun: self.xrun,
            round: self.round,
            appbufsz: self.appbufsz,
            __pad: self._pad,
            __magic: self._magic,
        }
    }
}
