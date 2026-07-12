use crate::error::{Error, Result};

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
