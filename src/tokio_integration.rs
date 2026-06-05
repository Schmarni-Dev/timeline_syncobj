use std::os::fd::AsRawFd;

use rustix::event::EventfdFlags;
use tokio::io::unix::AsyncFd;

use crate::{
    bindings::{DrmSyncobjEventFd, SyncobjEventFdFlags},
    timeline_syncobj::TimelineSyncObj,
};

impl TimelineSyncObj {
    fn wait_async_inner(
        &self,
        point: u64,
        flags: SyncobjEventFdFlags,
    ) -> rustix::io::Result<impl Future<Output = ()> + 'static + Send + Sync> {
        let event_fd = rustix::event::eventfd(0, EventfdFlags::NONBLOCK | EventfdFlags::CLOEXEC)?;
        unsafe {
            rustix::ioctl::ioctl(
                self.get_render_node(),
                DrmSyncobjEventFd {
                    handle: self.get_raw_handle(),
                    flags,
                    point,
                    fd: event_fd.as_raw_fd(),
                    _padding: 0,
                },
            )?;
        }
        // TODO: error handling
        let mut async_fd = AsyncFd::new(event_fd).unwrap();
        Ok(async move {
            // TODO: error handling
            let mut guard = async_fd.readable_mut().await.unwrap();
            _ = guard.try_io(|fd| {
                let mut buf = [0u8; 8];
                Ok(rustix::io::read(fd, &mut buf)?)
            });
        })
    }
    pub fn wait_async(
        &self,
        point: u64,
    ) -> rustix::io::Result<impl Future<Output = ()> + 'static + Send + Sync> {
        self.wait_async_inner(point, SyncobjEventFdFlags::empty())
    }
    pub fn wait_available_async(
        &self,
        point: u64,
    ) -> rustix::io::Result<impl Future<Output = ()> + 'static + Send + Sync> {
        self.wait_async_inner(point, SyncobjEventFdFlags::WAIT_AVAILABLE)
    }
}

#[tokio::test]
async fn async_wait() {
    let node = crate::render_node::DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let wait = tokio::spawn(obj.wait_async(16).unwrap());
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    assert!(!wait.is_finished());
    unsafe { obj.signal(32).unwrap() };
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    assert!(wait.is_finished());
    wait.await.unwrap();
}
