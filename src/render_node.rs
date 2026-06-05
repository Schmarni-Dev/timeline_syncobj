use std::{
    io::ErrorKind,
    os::fd::{AsFd, OwnedFd},
    sync::Arc,
};

use rustix::fs::{Mode, OFlags};

use crate::bindings::{DrmSyncobjHandleToFd, RawDrmSyncobjHandle, SyncobjHandleToFdFlags};

#[derive(Debug, Clone)]
pub struct DrmRenderNode {
    fd: Arc<OwnedFd>,
    pub(crate) direct_syncfile: bool,
}
impl DrmRenderNode {
    pub fn new(id: u64) -> rustix::io::Result<Self> {
        let path = format!("/dev/dri/renderD{}", id & 0xFF);
        let fd =
            rustix::fs::open(path, OFlags::RDWR | OFlags::CLOEXEC, Mode::empty()).map(Arc::new)?;
        let ret = unsafe {
            rustix::ioctl::ioctl(
                &fd,
                DrmSyncobjHandleToFd {
                    handle: RawDrmSyncobjHandle::NULL,
                    flags: SyncobjHandleToFdFlags::EXPORT_SYNC_FILE
                        | SyncobjHandleToFdFlags::TIMELINE,
                    fd: 0,
                    _padding: 0,
                    point: 1,
                },
            )
        };
        let direct_syncfile = match ret {
            Ok(_) => true,
            Err(err) if err.kind() == ErrorKind::NotFound => true,
            Err(err) if err.kind() == ErrorKind::InvalidInput => false,
            Err(_) => false,
        };
        Ok(Self {
            fd,
            direct_syncfile,
        })
    }
}
impl PartialEq for DrmRenderNode {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.fd, &other.fd)
    }
}
impl AsFd for DrmRenderNode {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.fd.as_fd()
    }
}
