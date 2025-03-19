use std::fs::File;
use std::io;

use std::os::twizzler::MetadataExt;

// A stable alternative to https://doc.rust-lang.org/stable/std/primitive.never.html
enum Never {}

pub struct MmapInner {
    handle: RawHandle,
    len: usize,
    off: usize,
}

use twizzler_rt_abi::object::MapFlags;
pub(crate) type = twizzler_rt_abi::object::ObjectHandle; 

impl MmapInner {
    fn new(id: u128, map_flags: MapFlags, len: usize, off: usize) -> io::Result<MmapInner> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "platform not supported",
        ))
    }

    pub fn map(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid(), MapFlags::READ, len, off)
    }

    pub fn map_exec(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid(), MapFlags::READ | MapFlags::EXEC, len, off)
    }

    pub fn map_mut(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid(), MapFlags::READ | MapFlags::WRITE, len, off)
    }

    pub fn map_copy(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid(), MapFlags::READ, len, off)
    }

    pub fn map_copy_read_only(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid(), MapFlags::READ, len, off)
    }

    pub fn map_anon(len: usize, _: bool, _: bool, _: Option<u8>) -> io::Result<MmapInner> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "operation not supported",
        ))
    }

    pub fn flush(&self, _: usize, _: usize) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "operation not supported",
        ))
    }

    pub fn flush_async(&self, _: usize, _: usize) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "operation not supported",
        ))
    }

    pub fn make_read_only(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub fn make_exec(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "operation not supported",
        ))
    }

    pub fn make_mut(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "operation not supported",
        ))
    }

    #[inline]
    pub fn ptr(&self) -> *const u8 {
        unsafe {
            self.handle.start().add(self.off)
        }
    }

    #[inline]
    pub fn mut_ptr(&mut self) -> *mut u8 {
        unsafe {
            self.handle.start().add(self.off)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

pub fn file_len(file: &File) -> io::Result<u64> {
    Ok(file.metadata()?.len())
}
