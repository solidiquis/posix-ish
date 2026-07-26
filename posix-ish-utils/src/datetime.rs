use crate::error::{Error, Result, ToPosixishResult};
use std::{mem, ptr};

pub fn to_local_dt(unix_secs: i64) -> Result<libc::tm> {
    let ts = to_time_t(unix_secs)?;
    let time_p = ptr::addr_of!(ts);

    let mut result = unsafe { mem::zeroed::<libc::tm>() };
    let result_p = ptr::addr_of_mut!(result);

    unsafe {
        if libc::localtime_r(time_p, result_p) == ptr::null_mut() {
            return Err(Error::io_error("failed to convert system time to local"));
        }
    }

    Ok(result)
}

pub fn to_utc_dt(unix_secs: i64) -> Result<libc::tm> {
    let ts = to_time_t(unix_secs)?;
    let time_p = ptr::addr_of!(ts);

    let mut result = unsafe { mem::zeroed::<libc::tm>() };
    let result_p = ptr::addr_of_mut!(result);

    unsafe {
        if libc::gmtime_r(time_p, result_p) == ptr::null_mut() {
            return Err(Error::io_error("failed to convert system time to UTC"));
        }
    }

    Ok(result)
}

fn to_time_t(unix_secs: i64) -> Result<libc::time_t> {
    libc::time_t::try_from(unix_secs).internal("timestamp doesn't fit in time_t")
}
