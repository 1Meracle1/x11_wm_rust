use std::{io::Error, os::fd::RawFd};

pub struct Epoll {
    epoll_fd: RawFd,
    events_capacity: usize,
    events: Vec<libc::epoll_event>,
}

impl Epoll {
    pub fn new(events_capacity: usize) -> Result<Self, std::io::Error> {
        debug_assert!(events_capacity > 0);
        let epoll_fd = unsafe { libc::epoll_create1(0) };
        if epoll_fd == -1 {
            return Err(Error::last_os_error());
        }
        let mut events = Vec::with_capacity(events_capacity);
        events.resize(events_capacity, libc::epoll_event { events: 0, u64: 0 });

        Ok(Self {
            epoll_fd,
            events_capacity,
            events,
        })
    }

    pub fn add_watch(&mut self, client_fd: RawFd) -> Result<(), std::io::Error> {
        assert!(!self.is_closed());
        let mut event = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: client_fd as u64,
        };
        let ctl_res =
            unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, client_fd, &mut event) };
        if ctl_res == -1 {
            let error = Error::last_os_error();
            self.close();
            return Err(error);
        }

        Ok(())
    }

    pub fn remove_watch(&mut self, client_fd: RawFd) -> Result<(), std::io::Error> {
        assert!(!self.is_closed());
        let ctl_res = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                libc::EPOLL_CTL_DEL,
                client_fd,
                std::ptr::null_mut(),
            )
        };
        if ctl_res == -1 {
            let error = Error::last_os_error();
            self.close();
            return Err(error);
        }

        Ok(())
    }

    pub fn wait(&mut self) -> Result<&Vec<libc::epoll_event>, std::io::Error> {
        assert!(!self.is_closed());
        self.events.clear();
        self.events.resize(
            self.events_capacity,
            libc::epoll_event { events: 0, u64: 0 },
        );
        let num_events = unsafe {
            libc::epoll_wait(
                self.epoll_fd,
                self.events.as_mut_ptr(),
                self.events_capacity as i32,
                -1,
            )
        };
        if num_events == -1 {
            let error = Error::last_os_error();
            self.close();
            return Err(error);
        }
        self.events
            .resize(num_events as usize, libc::epoll_event { events: 0, u64: 0 });
        Ok(&self.events)
    }

    pub fn close(&mut self) {
        if self.epoll_fd != -1 {
            unsafe { libc::close(self.epoll_fd) };
            self.epoll_fd = -1;
        }
    }

    pub fn is_closed(&self) -> bool {
        self.epoll_fd == -1
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        self.close();
    }
}
