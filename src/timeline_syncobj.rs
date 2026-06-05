use std::{
    os::fd::{AsFd, AsRawFd, OwnedFd},
    time::Duration,
};

use crate::{
    bindings::{
        DRM_CAP_SYNCOBJ, DRM_CAP_SYNCOBJ_TIMELINE, DrmGetCap, DrmSyncobjCreate, DrmSyncobjDestroy,
        DrmSyncobjFdToHandle, DrmSyncobjHandleToFd, DrmSyncobjTimelineQuery,
        DrmSyncobjTimelineSignal, DrmSyncobjTimelineWait, DrmSyncobjTransfer, RawDrmSyncobjHandle,
        SyncobjCreateFlags, SyncobjFdToHandleFlags, SyncobjHandleToFdFlags,
        SyncobjTimelineQueryFlags, SyncobjTimelineSignalFlags, SyncobjTransferFlags,
        SyncobjWaitFlags,
    },
    render_node::DrmRenderNode,
};

/// point 0 is a special case, start counting at 1
#[derive(Debug)]
pub struct TimelineSyncObj {
    handle: RawDrmSyncobjHandle,
    render_node: DrmRenderNode,
}

impl TimelineSyncObj {
    /// the point needs to be available for the export to succeed
    pub fn export_sync_file_point(&self, point: u64) -> rustix::io::Result<OwnedFd> {
        if self.render_node.direct_syncfile {
            unsafe {
                rustix::ioctl::ioctl(
                    &self.render_node,
                    DrmSyncobjHandleToFd {
                        handle: self.handle,
                        flags: SyncobjHandleToFdFlags::EXPORT_SYNC_FILE
                            | SyncobjHandleToFdFlags::TIMELINE,
                        fd: -1,
                        _padding: 0,
                        point,
                    },
                )
            }
        } else {
            let tmp = TimelineSyncObj::new(&self.render_node)?;
            self.transfer_point(&tmp, point, 0)?;
            unsafe {
                rustix::ioctl::ioctl(
                    &self.render_node,
                    DrmSyncobjHandleToFd {
                        handle: tmp.handle,
                        flags: SyncobjHandleToFdFlags::EXPORT_SYNC_FILE,
                        fd: -1,
                        _padding: 0,
                        point: 0,
                    },
                )
            }
        }
    }
    pub fn import_sync_file_point(
        &self,
        sync_file: impl AsFd,
        point: u64,
    ) -> rustix::io::Result<()> {
        let sync_file = sync_file.as_fd();
        if self.render_node.direct_syncfile {
            unsafe {
                rustix::ioctl::ioctl(
                    &self.render_node,
                    DrmSyncobjFdToHandle {
                        handle: self.handle,
                        flags: SyncobjFdToHandleFlags::IMPORT_SYNC_FILE
                            | SyncobjFdToHandleFlags::TIMELINE,
                        fd: sync_file.as_raw_fd(),
                        _padding: 0,
                        point,
                    },
                )?
            };
        } else {
            let tmp = TimelineSyncObj::new(&self.render_node)?;
            unsafe {
                rustix::ioctl::ioctl(
                    &self.render_node,
                    DrmSyncobjFdToHandle {
                        handle: self.handle,
                        flags: SyncobjFdToHandleFlags::IMPORT_SYNC_FILE,
                        fd: sync_file.as_raw_fd(),
                        _padding: 0,
                        point: 0,
                    },
                )?
            };
            tmp.transfer_point(self, 0, point)?;
        }
        Ok(())
    }

    fn query_point(&self, flags: SyncobjTimelineQueryFlags) -> rustix::io::Result<u64> {
        let handles: &[_] = &[self.handle];
        let points: &mut [_] = &mut [0u64];
        let points = unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjTimelineQuery {
                    handles: handles.as_ptr() as u64,
                    points: points.as_ptr() as u64,
                    count_handles: handles.len() as u32,
                    flags,
                },
            )?
        };
        Ok(points[0])
    }
    /// Returns the highest signaled point
    pub fn query_signaled_point(&self) -> rustix::io::Result<u64> {
        self.query_point(SyncobjTimelineQueryFlags::empty())
    }
    /// Returns the highest available point
    pub fn query_available_point(&self) -> rustix::io::Result<u64> {
        self.query_point(SyncobjTimelineQueryFlags::LAST_SUBMITTED)
    }

    /// the point needs to be available on this timeline for the transfer to succeed
    pub fn transfer_point(
        &self,
        to: &Self,
        src_point: u64,
        target_point: u64,
    ) -> rustix::io::Result<()> {
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjTransfer {
                    src_handle: self.get_raw_handle(),
                    dst_handle: to.get_raw_handle(),
                    src_point,
                    dst_point: target_point,
                    flags: SyncobjTransferFlags::empty(),
                    pad: 0,
                },
            )?
        };
        Ok(())
    }
    /// # Safety:
    /// if you did any gpu work make sure to only signal this after the work is actually complete,
    /// using something like a fence or VkTimelineSemaphore
    pub unsafe fn signal(&self, point: u64) -> rustix::io::Result<()> {
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjTimelineSignal {
                    handles: &raw const self.handle as u64,
                    points: [point].as_ptr() as u64,
                    count_handles: 1,
                    flags: SyncobjTimelineSignalFlags::empty(),
                },
            )?
        };
        Ok(())
    }
    fn blocking_wait_inner(
        &self,
        point: u64,
        timeout: Option<Duration>,
        flags: SyncobjWaitFlags,
    ) -> rustix::io::Result<()> {
        let handles: &[_] = &[self.handle];
        let points: &[_] = &[point];
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjTimelineWait {
                    handles: handles.as_ptr() as u64,
                    points: points.as_ptr() as u64,
                    timeout_nsec: timeout
                        .and_then(|v| v.as_nanos().try_into().ok())
                        .unwrap_or(i64::MAX),
                    count_handles: handles.len() as u32,
                    flags,
                    first_signaled: 0,
                    _padding: 0,
                    deadline_nsec: 0,
                },
            )?
        };
        Ok(())
    }

    pub fn blocking_wait(&self, point: u64, timeout: Option<Duration>) -> rustix::io::Result<()> {
        self.blocking_wait_inner(point, timeout, SyncobjWaitFlags::FOR_SUBMIT)
    }
    pub fn blocking_wait_available(
        &self,
        point: u64,
        timeout: Option<Duration>,
    ) -> rustix::io::Result<()> {
        self.blocking_wait_inner(point, timeout, SyncobjWaitFlags::AVAILABLE)
    }
}

impl TimelineSyncObj {
    pub fn new(render_node: &DrmRenderNode) -> rustix::io::Result<Self> {
        let (syncobj_cap, timeline_syncobj_cap) = unsafe {
            (
                rustix::ioctl::ioctl(
                    render_node,
                    DrmGetCap {
                        cap: DRM_CAP_SYNCOBJ,
                        value: 0,
                    },
                )?,
                rustix::ioctl::ioctl(
                    render_node,
                    DrmGetCap {
                        cap: DRM_CAP_SYNCOBJ_TIMELINE,
                        value: 0,
                    },
                )?,
            )
        };
        if syncobj_cap == 0 {
            panic!("syncobj not supported by drm driver");
        }
        if timeline_syncobj_cap == 0 {
            panic!("timeline_syncobj not supported by drm driver");
        }
        let handle = unsafe {
            rustix::ioctl::ioctl(
                render_node,
                DrmSyncobjCreate {
                    handle: 0,
                    flags: SyncobjCreateFlags::empty(),
                },
            )?
        };
        Ok(TimelineSyncObj {
            handle,
            render_node: render_node.clone(),
        })
    }
    pub fn import(render_node: &DrmRenderNode, fd: impl AsFd) -> rustix::io::Result<Self> {
        let fd = fd.as_fd();
        let handle = unsafe {
            rustix::ioctl::ioctl(
                render_node,
                DrmSyncobjFdToHandle {
                    handle: RawDrmSyncobjHandle::NULL,
                    flags: SyncobjFdToHandleFlags::empty(),
                    fd: fd.as_raw_fd(),
                    _padding: 0,
                    point: 0,
                },
            )?
        };
        Ok(TimelineSyncObj {
            handle,
            render_node: render_node.clone(),
        })
    }
    pub fn export(&self) -> rustix::io::Result<OwnedFd> {
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjHandleToFd {
                    handle: self.handle,
                    flags: SyncobjHandleToFdFlags::empty(),
                    fd: 0,
                    _padding: 0,
                    point: 0,
                },
            )
        }
    }
    /// # Safety:
    /// Don't destroy the raw handle
    pub unsafe fn get_raw_handle(&self) -> RawDrmSyncobjHandle {
        self.handle
    }
    pub fn get_render_node(&self) -> &DrmRenderNode {
        &self.render_node
    }
}
impl Drop for TimelineSyncObj {
    fn drop(&mut self) {
        unsafe {
            _ = rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjDestroy {
                    handle: self.handle,
                    _padding: 0,
                },
            );
        }
    }
}

#[test]
fn point_signaling() {
    let node = crate::render_node::DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    assert_eq!(obj.query_signaled_point(), Ok(0));
    unsafe { obj.signal(32).unwrap() };
    assert_eq!(obj.query_signaled_point(), Ok(32));
}

#[test]
fn point_transfer() {
    let node = crate::render_node::DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj2 = TimelineSyncObj::new(&node).expect("failed to create second syncojb");
    assert_eq!(obj2.query_signaled_point(), Ok(0));
    unsafe { obj.signal(32).unwrap() };
    obj.transfer_point(&obj2, 32, 1).unwrap();
    assert_eq!(obj.query_signaled_point(), Ok(32));
    assert_eq!(obj2.query_signaled_point(), Ok(1));
}

#[test]
fn sharing() {
    let node = crate::render_node::DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj_fd = obj.export().expect("failed to export syncobj");
    let obj2 = TimelineSyncObj::import(&node, obj_fd).expect("failed to import syncobj");
    assert_eq!(obj.query_signaled_point(), Ok(0));
    assert_eq!(obj2.query_signaled_point(), Ok(0));
    unsafe { obj.signal(32).unwrap() };
    assert_eq!(obj.query_signaled_point(), Ok(32));
    assert_eq!(obj2.query_signaled_point(), Ok(32));
}

#[test]
fn sync_file_exporting() {
    use std::io::ErrorKind;
    let node = crate::render_node::DrmRenderNode::new(128).expect("failed to open render node");
    let obj = TimelineSyncObj::new(&node).expect("failed to create syncojb");
    let obj2 = TimelineSyncObj::new(&node).expect("failed to create second syncobj");
    assert!(
        obj.export_sync_file_point(32)
            .is_err_and(|err| err.kind() == ErrorKind::InvalidInput)
    );
    unsafe { obj.signal(32).unwrap() };
    let export = obj.export_sync_file_point(32).unwrap();
    obj2.import_sync_file_point(export, 32).unwrap();
    assert_eq!(obj2.query_signaled_point(), Ok(32));
}
