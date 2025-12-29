use std::{
    os::fd::{IntoRawFd, OwnedFd},
    time::Duration,
};

use crate::{
    bindings::{
        DrmSyncobjCreate, DrmSyncobjDestroy, DrmSyncobjFdToHandle, DrmSyncobjHandleToFd,
        DrmSyncobjTimelineSignal, DrmSyncobjTimelineWait, RawDrmSyncobjHandle, SyncobjCreateFlags,
        SyncobjFdToHandleFlags, SyncobjHandleToFdFlags, SyncobjTimelineSignalFlags,
        SyncobjWaitFlags,
    },
    render_node::DrmRenderNode,
};

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
    pub fn import_sync_file_point(&self, sync_file: OwnedFd, point: u64) -> rustix::io::Result<()> {
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjFdToHandle {
                    handle: self.handle,
                    flags: SyncobjFdToHandleFlags::IMPORT_SYNC_FILE,
                    fd: sync_file.into_raw_fd(),
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
        unsafe {
            rustix::ioctl::ioctl(
                &self.render_node,
                DrmSyncobjTimelineWait {
                    handles: &raw const self.handle as u64,
                    points: [point].as_ptr() as u64,
                    timeout_nsec: timeout
                        // TODO: return an error here
                        .and_then(|v| v.as_nanos().try_into().ok())
                        .unwrap_or(i64::MAX),
                    count_handles: 1,
                    flags: SyncobjWaitFlags::empty(),
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
    pub fn import(render_node: &DrmRenderNode, fd: OwnedFd) -> rustix::io::Result<Self> {
        let handle = unsafe {
            rustix::ioctl::ioctl(
                render_node,
                DrmSyncobjFdToHandle {
                    handle: RawDrmSyncobjHandle::NULL,
                    flags: SyncobjFdToHandleFlags::TIMELINE,
                    fd: fd.into_raw_fd(),
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
