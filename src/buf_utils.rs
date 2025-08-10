use nanoid::nanoid;
use std::{
    ffi::c_void,
    os::fd::{AsFd, OwnedFd},
    ptr::NonNull,
};
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface};

use nix::{
    fcntl::OFlag,
    sys::{
        mman::{ProtFlags, shm_open, shm_unlink},
        stat::Mode,
    },
    unistd::ftruncate,
};

#[derive(Debug)]
pub struct Surface {
    pub width: usize,
    pub height: usize,
    pub wl_surface: WlSurface,
    pub buf: MMappedBuf,
    pub wl_buf: Option<WlBuffer>,
}

impl Surface {
    pub fn init_buf(&mut self, width: usize, height: usize) {
        self.buf = allocate_shm_buffer(width * height * 4 * 2);
    }
}

#[derive(Debug)]
pub struct MMappedBuf {
    pub fd: OwnedFd,
    ptr: NonNull<c_void>,
    shm_name: String,
    pub len: usize,
}

impl MMappedBuf {
    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *mut u8, self.len) }
    }

    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut u8, self.len) }
    }
}

impl Drop for MMappedBuf {
    fn drop(&mut self) {
        unsafe {
            let _ = nix::sys::mman::munmap(self.ptr, self.len);
        };
        shm_unlink(self.shm_name.as_str().into()).unwrap();
    }
}

pub fn allocate_shm_buffer<'a>(len: usize) -> MMappedBuf {
    let name = format!("/nomouse-buf-{}", nanoid!());
    let fd = shm_open(
        name.as_str().into(),
        OFlag::O_CREAT | OFlag::O_RDWR | OFlag::O_EXCL,
        Mode::from_bits(0600).unwrap(),
    )
    .unwrap();
    ftruncate(&fd, len as i64).unwrap();
    let ptr = unsafe {
        nix::sys::mman::mmap(
            None,
            len.try_into().unwrap(),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            nix::sys::mman::MapFlags::MAP_SHARED,
            fd.as_fd(),
            0,
        )
        .unwrap()
    };
    MMappedBuf {
        shm_name: name,
        fd,
        ptr,
        len,
    }
}
