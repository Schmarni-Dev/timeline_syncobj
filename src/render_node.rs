use std::{
    io::ErrorKind,
    os::fd::{AsFd, OwnedFd},
    sync::Arc,
};

use rustix::fs::{Mode, OFlags};
use thiserror::Error;

use crate::{
    bindings::{
        DrmSyncobjHandleToFd, DrmSyncobjTimelineQuery, RawDrmSyncobjHandle, SyncobjHandleToFdFlags,
        SyncobjTimelineQueryFlags,
    },
    timeline_syncobj::TimelineSyncObj,
};

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
                    fd: -1,
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
impl DrmRenderNode {
    /// Returns the highest signaled point for each [`TimelineSyncObj`]
    pub fn query_signaled_points(
        &self,
        objs: &[&TimelineSyncObj],
    ) -> Result<Vec<u64>, ManySyncobjsError> {
        self.query_points(objs, SyncobjTimelineQueryFlags::empty())
    }
    /// Returns the highest available point for each [`TimelineSyncObj`]
    pub fn query_available_points(
        &self,
        objs: &[&TimelineSyncObj],
    ) -> Result<Vec<u64>, ManySyncobjsError> {
        self.query_points(objs, SyncobjTimelineQueryFlags::LAST_SUBMITTED)
    }
    fn query_points(
        &self,
        objs: &[&TimelineSyncObj],
        flags: SyncobjTimelineQueryFlags,
    ) -> Result<Vec<u64>, ManySyncobjsError> {
        if !objs.iter().all(|v| self == v.get_render_node()) {
            return Err(ManySyncobjsError::MismatchedSyncObjs);
        }
        let handles: Vec<RawDrmSyncobjHandle> =
            objs.iter().map(|v| unsafe { v.get_raw_handle() }).collect();
        let mut points: Vec<u64> = vec![0u64; handles.len()];
        let points: &mut [u64] = &mut points;
        let points = unsafe {
            rustix::ioctl::ioctl(
                self,
                DrmSyncobjTimelineQuery {
                    handles: handles.as_ptr() as u64,
                    points: points.as_ptr() as u64,
                    count_handles: handles.len() as u32,
                    flags,
                },
            )?
        };
        Ok(points)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ManySyncobjsError {
    #[error("At least one provided TimelineSyncObj is not owned by this DrmRenderNode")]
    MismatchedSyncObjs,
    #[error("Underlying call failed: {0}")]
    CallError(#[from] rustix::io::Errno),
}

#[test]
fn point_signaling() {
    let node = crate::render_node::DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj2 = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    assert_eq!(node.query_signaled_points(&[&obj, &obj2]), Ok(vec![0, 0]));
    unsafe { obj.signal(32).unwrap() };
    unsafe { obj2.signal(64).unwrap() };
    assert_eq!(node.query_signaled_points(&[&obj, &obj2]), Ok(vec![32, 64]));
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
