use std::{
    os::fd::{AsFd, OwnedFd},
    sync::Arc,
};

use rustix::fs::{Mode, OFlags};

#[derive(Debug, Clone)]
pub struct DrmRenderNode(Arc<OwnedFd>);
impl DrmRenderNode {
    pub fn new(id: u64) -> rustix::io::Result<Self> {
        let path = format!("/dev/dri/renderD{}", id & 0xFF);
        rustix::fs::open(path, OFlags::RDWR | OFlags::CLOEXEC, Mode::empty())
            .map(Arc::new)
            .map(Self)
    }
}
impl PartialEq for DrmRenderNode {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl AsFd for DrmRenderNode {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.0.as_fd()
    }
}
