use crate::error::{Error, Result};
use std::{
    env,
    io::{IsTerminal, stdout},
};

/// https://man7.org/linux/man-pages/man2/TIOCSWINSZ.2const.html
pub fn get_winsize() -> Result<libc::winsize> {
    let mut winsize = unsafe { std::mem::zeroed::<libc::winsize>() };
    let winsize_mut_ptr = std::ptr::addr_of_mut!(winsize);

    let status = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, winsize_mut_ptr) };
    if status == -1 {
        return Err(Error::io_error("failed to get TTY window size"));
    }
    Ok(winsize)
}

/// https://web.archive.org/web/20260616201813/https://no-color.org/
pub fn enable_color() -> bool {
    if !stdout().is_terminal() {
        return false;
    }
    !env::var("NO_COLOR").is_ok_and(|v| !v.is_empty())
}
