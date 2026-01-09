use core::slice;
use std::os::fd::{FromRawFd, OwnedFd, RawFd};

use bitflags::bitflags;
use derive_more::Deref;
use rustix::ioctl::{Ioctl, opcode::read_write};

pub const DRM_IOCTL_BASE: u8 = b'd';

pub const DRM_CAP_SYNCOBJ: DrmCap = DrmCap(0x13);
pub const DRM_CAP_SYNCOBJ_TIMELINE: DrmCap = DrmCap(0x14);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deref)]
#[repr(transparent)]
pub struct DrmCap(u64);
#[repr(C)]
pub struct DrmGetCap {
    pub cap: DrmCap,
    pub value: u64,
}

bitflags! {
    #[repr(transparent)]
    pub struct SyncobjCreateFlags: u32 {
        const CREATE_SIGNALED = 1 << 0;

        const _ = !0;
    }
}

#[repr(C)]
pub struct DrmSyncobjCreate {
    pub handle: u32,
    pub flags: SyncobjCreateFlags,
}

#[repr(C)]
pub struct DrmSyncobjDestroy {
    pub handle: RawDrmSyncobjHandle,
    pub _padding: u32,
}

bitflags! {
    #[repr(transparent)]
    pub struct SyncobjFdToHandleFlags: u32 {
        const IMPORT_SYNC_FILE = 1 << 0;
        const TIMELINE = 1 << 1;

        const _ = !0;
    }
}
bitflags! {
    #[repr(transparent)]
    pub struct SyncobjHandleToFdFlags: u32 {
        const EXPORT_SYNC_FILE = 1 << 0;
        const TIMELINE = 1 << 1;

        const _ = !0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deref)]
#[repr(transparent)]
pub struct RawDrmSyncobjHandle(u32);
impl RawDrmSyncobjHandle {
    pub const NULL: Self = Self(0);
}

#[repr(C)]
pub struct DrmSyncobjHandleToFd {
    pub handle: RawDrmSyncobjHandle,
    pub flags: SyncobjHandleToFdFlags,
    pub fd: RawFd,
    pub _padding: u32,
    pub point: u64,
}
#[repr(C)]
pub struct DrmSyncobjFdToHandle {
    pub handle: RawDrmSyncobjHandle,
    pub flags: SyncobjFdToHandleFlags,
    pub fd: RawFd,
    pub _padding: u32,
    pub point: u64,
}
bitflags! {
    #[repr(transparent)]
    pub struct SyncobjTransferFlags: u32 {

        const _ = !0;
    }
}
#[repr(C)]
pub struct DrmSyncobjTransfer {
    pub src_handle: RawDrmSyncobjHandle,
    pub dst_handle: RawDrmSyncobjHandle,
    pub src_point: u64,
    pub dst_point: u64,
    pub flags: SyncobjTransferFlags,
    pub pad: u32,
}

bitflags! {
    #[repr(transparent)]
    pub struct SyncobjWaitFlags: u32 {
        const ALL = 1 << 0;
        const FOR_SUBMIT = 1 << 1;
        const AVAILABLE = 1 << 2;
        const DEADLINE = 1 << 3;

        const _ = !0;
    }
}
#[repr(C)]
pub struct DrmSyncobjWait {
    pub handles: u64,
    pub timeout_nsec: i64,
    pub count_handles: u32,
    pub flags: SyncobjWaitFlags,
    pub first_signaled: u32,
    pub _padding: u32,
    pub deadline_nsec: u64,
}
#[repr(C)]
pub struct DrmSyncobjTimelineWait {
    pub handles: u64,
    pub points: u64,
    pub timeout_nsec: i64,
    pub count_handles: u32,
    pub flags: SyncobjWaitFlags,
    pub first_signaled: u32,
    pub _padding: u32,
    pub deadline_nsec: u64,
}
bitflags! {
    #[repr(transparent)]
    pub struct SyncobjEventFdFlags: u32 {
        const WAIT_AVAILABLE = 1 << 0;

        const _ = !0;
    }
}
#[repr(C)]
pub struct DrmSyncobjEventFd {
    pub handle: RawDrmSyncobjHandle,
    // TODO: figure out what type this should be, maybe wait flags?
    pub flags: SyncobjEventFdFlags,
    pub point: u64,
    pub fd: RawFd,
    pub _padding: u32,
}

#[repr(C)]
pub struct DrmSyncobjReset {
    pub handles: u64,
    pub count_handles: u32,
    pub pad: u32,
}
#[repr(C)]
pub struct DrmSyncobjSignal {
    pub handles: u64,
    pub count_handles: u32,
    pub pad: u32,
}

bitflags! {
    #[repr(transparent)]
    pub struct SyncobjTimelineQueryFlags: u32 {
        const LAST_SUBMITTED = 1 << 0;

        const _ = !0;
    }
}
bitflags! {
    #[repr(transparent)]
    pub struct SyncobjTimelineSignalFlags: u32 {
        // const LAST_SUBMITTED = 1 << 0;

        const _ = !0;
    }
}
#[repr(C)]
pub struct DrmSyncobjTimelineSignal {
    pub handles: u64,
    pub points: u64,
    pub count_handles: u32,
    pub flags: SyncobjTimelineSignalFlags,
}
#[repr(C)]
pub struct DrmSyncobjTimelineQuery {
    pub handles: u64,
    pub points: u64,
    pub count_handles: u32,
    pub flags: SyncobjTimelineQueryFlags,
}
unsafe impl Ioctl for DrmSyncobjCreate {
    type Output = RawDrmSyncobjHandle;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xBF)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        Ok(RawDrmSyncobjHandle(v.handle))
    }
}
unsafe impl Ioctl for DrmSyncobjDestroy {
    type Output = ();

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xC0)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        _extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
unsafe impl Ioctl for DrmSyncobjHandleToFd {
    type Output = OwnedFd;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xC1)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        Ok(unsafe { OwnedFd::from_raw_fd(v.fd) })
    }
}
unsafe impl Ioctl for DrmSyncobjFdToHandle {
    type Output = RawDrmSyncobjHandle;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xC2)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        Ok(v.handle)
    }
}
unsafe impl Ioctl for DrmSyncobjWait {
    type Output = Option<u32>;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xC3)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        Ok((!v.flags.contains(SyncobjWaitFlags::ALL)).then(|| v.first_signaled))
    }
}
unsafe impl Ioctl for DrmSyncobjReset {
    type Output = ();

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xC4)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        _extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
unsafe impl Ioctl for DrmSyncobjSignal {
    type Output = ();

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xC5)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        _extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
unsafe impl Ioctl for DrmSyncobjTimelineWait {
    type Output = Option<u32>;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xCA)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        Ok((!v.flags.contains(SyncobjWaitFlags::ALL)).then(|| v.first_signaled))
    }
}
unsafe impl Ioctl for DrmSyncobjTimelineQuery {
    type Output = Vec<u64>;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xCB)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        // really unsure about this one
        let array = v.points as *const u64;
        let slice = unsafe { slice::from_raw_parts(array, v.count_handles as usize) };
        Ok(slice.to_vec())
    }
}
unsafe impl Ioctl for DrmSyncobjTransfer {
    type Output = ();

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xCC)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        _extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
unsafe impl Ioctl for DrmSyncobjTimelineSignal {
    type Output = ();

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xCD)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        _extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
unsafe impl Ioctl for DrmSyncobjEventFd {
    type Output = ();

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0xCF)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        _extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
unsafe impl Ioctl for DrmGetCap {
    type Output = u64;

    const IS_MUTATING: bool = true;

    fn opcode(&self) -> rustix::ioctl::Opcode {
        read_write::<Self>(DRM_IOCTL_BASE, 0x0C)
    }

    fn as_ptr(&mut self) -> *mut rustix::ffi::c_void {
        self as *mut _ as *mut _
    }

    unsafe fn output_from_ptr(
        _out: rustix::ioctl::IoctlOutput,
        extract_output: *mut rustix::ffi::c_void,
    ) -> rustix::io::Result<Self::Output> {
        let ptr = extract_output as *mut Self;
        let v = unsafe { &(*ptr) };
        Ok(v.value)
    }
}
