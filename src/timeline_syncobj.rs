use std::{
    os::fd::{AsRawFd, BorrowedFd, OwnedFd},
    time::Duration,
};

use crate::{
    bindings::{
        DRM_CAP_SYNCOBJ, DRM_CAP_SYNCOBJ_TIMELINE, DrmGetCap, DrmSyncobjCreate, DrmSyncobjDestroy,
        DrmSyncobjFdToHandle, DrmSyncobjHandleToFd, DrmSyncobjTimelineSignal,
        DrmSyncobjTimelineWait, RawDrmSyncobjHandle, SyncobjCreateFlags, SyncobjFdToHandleFlags,
        SyncobjHandleToFdFlags, SyncobjTimelineSignalFlags, SyncobjWaitFlags,
    },
    render_node::DrmRenderNode,
};

#[derive(Debug)]
pub struct TimelineSyncObj {
    handle: RawDrmSyncobjHandle,
    render_node: DrmRenderNode,
}

impl TimelineSyncObj {
    pub fn export_sync_file_point(&self, point: u64) -> rustix::io::Result<OwnedFd> {
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjHandleToFd {
                    handle: self.handle,
                    flags: SyncobjHandleToFdFlags::EXPORT_SYNC_FILE,
                    fd: 0,
                    _padding: 0,
                    point,
                },
            )
        }
    }
    pub fn import_sync_file_point(
        &self,
        sync_file: BorrowedFd,
        point: u64,
    ) -> rustix::io::Result<()> {
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
        Ok(())
    }
    /// # Safety:
    /// if you did any gpu work related to this syncobj make sure to sync it to the cpu first,
    /// using a fence or VkTimelineSemaphore
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
    pub fn blocking_wait(&self, point: u64, timeout: Option<Duration>) -> rustix::io::Result<()> {
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
                    flags: SyncobjWaitFlags::AVAILABLE,
                    first_signaled: 0,
                    _padding: 0,
                    deadline_nsec: 0,
                },
            )?
        };
        Ok(())
    }
}

impl TimelineSyncObj {
    pub fn create(render_node: &DrmRenderNode) -> rustix::io::Result<Self> {
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
    pub fn import(render_node: &DrmRenderNode, fd: BorrowedFd) -> rustix::io::Result<Self> {
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
                    flags: SyncobjHandleToFdFlags::TIMELINE,
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
