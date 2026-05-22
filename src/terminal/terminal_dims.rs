use super::lib_c::LibCErr;
use std::{
    os::raw::{c_int, c_ulong},
    sync::atomic::{AtomicBool, Ordering},
};
const SIGWINCH: c_int = 28;
const STDOUT_FILENO: c_int = 1;
const TIOCGWINSZ: c_ulong = 0x5413;

pub(super) static TERM_SIZE_CHANGED: AtomicBool = AtomicBool::new(true);
static IS_TERM_SIZE_INIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    fn signal(sig: c_int, handler: extern "C" fn(c_int)) -> LibCErr;
    fn ioctl(fd: c_int, request: c_ulong, ...) -> LibCErr;
}

extern "C" fn handle_winch(_sig: c_int) {
    TERM_SIZE_CHANGED.swap(true, Ordering::Relaxed);
}

impl From<LibCErr> for TermSizeErr {
    fn from(value: LibCErr) -> Self {
        Self::LibCErr(value)
    }
}

#[derive(Debug)]
pub enum TermSizeErr {
    SingletonAlreadyExists,
    LibCErr(LibCErr),
}

type TermSizeRes<T> = Result<T, TermSizeErr>;
#[repr(C)]
#[derive(Debug)]
pub struct TerminalDims {
    pub rows: u16,
    pub cols: u16,
    pub pixels_x: u16,
    pub pixels_y: u16,
}

impl TerminalDims {
    pub fn new() -> TermSizeRes<TerminalDims> {
        let mut term_size = TerminalDims {
            rows: 0,
            cols: 0,
            pixels_x: 0,
            pixels_y: 0,
        };

        term_size.update_screen_size()?;
        let already_exists = IS_TERM_SIZE_INIT.swap(true, Ordering::Relaxed);
        if already_exists {
            return Err(TermSizeErr::SingletonAlreadyExists);
        }
        Self::init_size_listener()?;

        Ok(term_size)
    }

    pub fn update_screen_size(&mut self) -> TermSizeRes<()> {
        unsafe {
            ioctl(STDOUT_FILENO, TIOCGWINSZ, self).into_result()?;
        }

        Ok(())
    }

    fn init_size_listener() -> TermSizeRes<()> {
        unsafe {
            signal(SIGWINCH, handle_winch).into_result()?;
        }

        Ok(())
    }
}
