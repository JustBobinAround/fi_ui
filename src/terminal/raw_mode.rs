use super::lib_c::LibCErr;
use std::ffi::{c_int, c_uchar, c_uint, c_void};
use std::{
    marker::PhantomData,
    os::unix::{
        io::{IntoRawFd, RawFd},
        prelude::AsRawFd,
    },
    sync::{Arc, Mutex, OnceLock},
};

pub type CSizeT = usize;
pub type CSSizeT = isize;

static TERMINAL_MODE_PRIOR_RAW_MODE: OnceLock<Arc<Mutex<Option<Termios>>>> = OnceLock::new();
const STDIN_FILENO: c_int = 0;
const TCSANOW: c_int = 0;

#[repr(C)]
#[derive(Clone)]
pub struct Termios {
    pub c_iflag: c_uint,
    pub c_oflag: c_uint,
    pub c_cflag: c_uint,
    pub c_lflag: c_uint,
    pub c_line: c_uchar,
    pub c_cc: [c_uchar; 32],
    pub c_ispeed: c_uint,
    pub c_ospeed: c_uint,
}

unsafe extern "C" {
    fn cfmakeraw(termios: *mut Termios);
    fn close(fd: c_int) -> LibCErr;
    fn isatty(fd: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: CSizeT) -> CSSizeT;
    fn tcgetattr(fd: c_int, termios: *mut Termios) -> LibCErr;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios: *const Termios) -> LibCErr;
}

pub struct FileDesc<'a> {
    fd: RawFd,
    close_on_drop: bool,
    phantom: PhantomData<&'a ()>,
}

impl<'a> FileDesc<'a> {
    pub fn from_raw(fd: RawFd, close_on_drop: bool) -> FileDesc<'static> {
        FileDesc {
            fd,
            close_on_drop,
            phantom: PhantomData,
        }
    }
    pub fn read(&self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let result = unsafe {
            read(
                self.fd,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as CSizeT,
            )
        };

        if result < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(result as usize)
        }
    }

    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl<'a> Drop for FileDesc<'a> {
    fn drop(&mut self) {
        if self.close_on_drop {
            let _ = unsafe { close(self.fd) };
        }
    }
}

pub fn tty_fd() -> std::io::Result<FileDesc<'static>> {
    let (fd, close_on_drop) = if unsafe { isatty(STDIN_FILENO) == 1 } {
        (STDIN_FILENO, false)
    } else {
        (
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")?
                .into_raw_fd(),
            true,
        )
    };

    Ok(FileDesc::from_raw(fd, close_on_drop))
}

fn set_terminal_attr(fd: RawFd, termios: &Termios) -> std::io::Result<()> {
    let res = unsafe { tcsetattr(fd, TCSANOW, termios) };

    res.into_result()
        .map_err(|_| std::io::Error::last_os_error())?;

    Ok(())
}

fn get_terminal_attr(fd: RawFd) -> std::io::Result<Termios> {
    unsafe {
        let mut termios = std::mem::zeroed();
        tcgetattr(fd, &mut termios)
            .into_result()
            .map_err(|_| std::io::Error::last_os_error())?;
        Ok(termios)
    }
}

fn raw_terminal_attr(termios: &mut Termios) {
    unsafe { cfmakeraw(termios) }
}

pub fn enable_raw_mode() -> std::io::Result<()> {
    let original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.get_or_init(|| Arc::new(Mutex::new(None)));
    match original_mode.lock() {
        Ok(mut original_mode) => {
            let tty = tty_fd()?;
            let fd = tty.raw_fd();
            let mut ios = get_terminal_attr(fd)?;
            let original_mode_ios = ios.clone();
            raw_terminal_attr(&mut ios);
            set_terminal_attr(fd, &ios)?;
            *original_mode = Some(original_mode_ios);
        }

        Err(_) => {}
    }

    Ok(())
}

pub fn disable_raw_mode() -> std::io::Result<()> {
    match TERMINAL_MODE_PRIOR_RAW_MODE.get() {
        Some(tmprm) => match tmprm.lock() {
            Ok(mut tmprm) => match tmprm.as_ref() {
                Some(original_mode_ios) => {
                    let tty = tty_fd()?;
                    set_terminal_attr(tty.raw_fd(), original_mode_ios)?;
                    *tmprm = None;
                }
                None => {}
            },
            Err(_) => {}
        },
        None => {}
    };
    Ok(())
}
