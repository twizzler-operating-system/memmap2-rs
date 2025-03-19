use std::fs::File;
use std::io;

use std::os::twizzler::fs::MetadataExt;
use twizzler_rt_abi::object::{ObjectHandle, MapFlags};

// A stable alternative to https://doc.rust-lang.org/stable/std/primitive.never.html
enum Never {}

pub struct MmapInner {
    handle: ObjectHandle,
    len: usize,
    off: usize,
}


impl MmapInner {
    fn new(id: u128, map_flags: MapFlags, len: usize, off: u64) -> io::Result<MmapInner> {
        let handle = twizzler_rt_abi::object::twz_rt_map_object(id.into(), map_flags).map_err(|_| io::Error::new(
            io::ErrorKind::Other,
            "mmap failed",
        ))?;
        Ok(Self {
            // TODO: get this from twizzler crate
            handle, len, off: off as usize + 0x1000,
        })
    }

    pub fn map(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid().into(), MapFlags::READ, len, off)
    }

    pub fn map_exec(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid().into(), MapFlags::READ | MapFlags::EXEC, len, off)
    }

    pub fn map_mut(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid().into(), MapFlags::READ | MapFlags::WRITE, len, off)
    }

    pub fn map_copy(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid().into(), MapFlags::READ, len, off)
    }

    pub fn map_copy_read_only(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid().into(), MapFlags::READ, len, off)
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
