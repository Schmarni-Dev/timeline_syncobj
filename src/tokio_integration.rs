use std::os::fd::AsRawFd;

use rustix::event::EventfdFlags;
use tokio::io::unix::AsyncFd;

use crate::{
    bindings::{DrmSyncobjEventFd, SyncobjEventFdFlags},
    timeline_syncobj::TimelineSyncObj,
};

impl TimelineSyncObj {
    pub fn wait_async(
        &self,
        point: u64,
    ) -> rustix::io::Result<impl Future + 'static + Send + Sync> {
        let event_fd = rustix::event::eventfd(0, EventfdFlags::NONBLOCK | EventfdFlags::NONBLOCK)?;
        unsafe {
            rustix::ioctl::ioctl(
                self.get_render_node(),
                DrmSyncobjEventFd {
                    handle: self.get_raw_handle(),
                    flags: SyncobjEventFdFlags::empty(),
                    point,
                    fd: event_fd.as_raw_fd(),
                    _padding: 0,
                },
            )?;
        }
        // TODO: error handling
        let async_fd = AsyncFd::new(event_fd).unwrap();
        Ok(async move {
            // TODO: error handling
            let guard = async_fd.readable().await.unwrap();
            guard
                .try_io(|_| {
                    let mut buf = [0u8; 8];
                    rustix::io::read(async_fd.as_raw_fd(), &mut buf)
                })
                .unwrap();
        })
    }
}
