use libc::IN_NONBLOCK;
use std::{
    ffi::CString,
    os::{fd::RawFd, unix::ffi::OsStrExt},
    path::Path,
};

#[derive(Debug)]
pub enum Error<'a> {
    OsError(std::io::Error),
    NonExistentPathProvided {
        path: &'a str,
    },
    FailedToConvertPathStringToCString {
        path: &'a str,
        e: std::ffi::NulError,
    },
}

#[derive(Debug)]
pub enum InotifyEvent {
    Modify,
    CloseWrite,
    Unknown,
}

pub struct Inotify {
    pub fd: RawFd,
}

impl<'a> Inotify {
    pub fn new() -> Result<Self, Error<'a>> {
        let fd = unsafe { libc::inotify_init1(IN_NONBLOCK) };
        if fd == -1 {
            return Err(Error::OsError(std::io::Error::last_os_error()));
        }
        Ok(Self { fd })
    }

    pub fn add_watch(&'a mut self, filepath: &'a str) -> Result<(), Error<'a>> {
        let path = Path::new(filepath);
        if !path.exists() {
            return Err(Error::NonExistentPathProvided { path: filepath });
        }
        let bytes = path.as_os_str().as_bytes();
        let path_cstr = CString::new(bytes)
            .map_err(|e| Error::FailedToConvertPathStringToCString { path: filepath, e })?;
        let res = unsafe {
            libc::inotify_add_watch(
                self.fd,
                path_cstr.as_ptr(),
                libc::IN_MODIFY | libc::IN_CLOSE_WRITE,
            )
        };
        if res == -1 {
            Err(Error::OsError(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn read_event(&'a mut self) -> Result<InotifyEvent, Error<'a>> {
        let mut buffer = [0u8; 4096];
        let len = unsafe {
            libc::read(
                self.fd,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                buffer.len(),
            )
        };
        if len < 0 {
            Err(Error::OsError(std::io::Error::last_os_error()))
        } else {
            let event = unsafe { *(buffer.as_ptr() as *const libc::inotify_event) };
            if event.mask & libc::IN_MODIFY == 0 {
                Ok(InotifyEvent::Modify)
            } else if event.mask & libc::IN_CLOSE_WRITE == 0 {
                Ok(InotifyEvent::CloseWrite)
            } else {
                Ok(InotifyEvent::Unknown)
            }
        }
    }
}

impl Drop for Inotify {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
