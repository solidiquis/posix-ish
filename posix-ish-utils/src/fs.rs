use crate::error::{Error, Result};
use std::ffi::CStr;

pub fn get_owner(uid: u32) -> Result<String> {
    unsafe {
        let ptr = libc::getpwuid(libc::uid_t::from(uid));

        if ptr.is_null() {
            return Err(Error::io_error("failed to get passwd"));
        }
        let passwd = *ptr;

        if passwd.pw_name.is_null() {
            return Err(Error::io_error("pw_name field was null"));
        }
        let c_str = CStr::from_ptr(passwd.pw_name);

        Ok(c_str.to_string_lossy().into_owned())
    }
}
pub fn get_group(gid: u32) -> Result<String> {
    unsafe {
        let ptr = libc::getgrgid(libc::gid_t::from(gid));

        if ptr.is_null() {
            return Err(Error::io_error("failed to get group"));
        }
        let group = *ptr;

        if group.gr_name.is_null() {
            return Err(Error::io_error("gr_name field was null"));
        }

        let c_str = CStr::from_ptr(group.gr_name);

        Ok(c_str.to_string_lossy().into_owned())
    }
}
