use std::{fs::File, io, os::twizzler::fs::MetadataExt, ptr::addr_of_mut, sync::atomic::AtomicU64};

use twizzler_rt_abi::{
    bindings::{sync_info, SYNC_FLAG_ASYNC_DURABLE, SYNC_FLAG_DURABLE},
    object::{
        MapFlags, ObjID, ObjectCmd, ObjectCreate, ObjectCreateFlags, ObjectHandle, ObjectSource,
        MAX_SIZE, NULLPAGE_SIZE,
    },
};

// A stable alternative to https://doc.rust-lang.org/stable/std/primitive.never.html
#[allow(unused)]
enum Never {}

pub struct MmapInner {
    handle: ObjectHandle,
    len: usize,
    off: usize,
}

fn other_err(msg: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

fn check_range(off: usize, len: usize) -> io::Result<()> {
    // File data lives at NULLPAGE_SIZE within the object (same layout RawFile reads); the top
    // page is metadata.
    if off
        .checked_add(len)
        .is_none_or(|end| end > MAX_SIZE - NULLPAGE_SIZE * 2)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "mmap range exceeds object data size",
        ));
    }
    Ok(())
}

fn create_object(sources: &[ObjectSource]) -> io::Result<ObjID> {
    // DELETE = delete once the last mapping goes away, so dropping the handle is the cleanup.
    let spec = ObjectCreate {
        flags: ObjectCreateFlags::DELETE,
        ..Default::default()
    };
    let sources: Vec<twizzler_rt_abi::bindings::object_source> =
        sources.iter().map(|s| (*s).into()).collect();
    let res = unsafe {
        twizzler_rt_abi::bindings::twz_rt_create_object(
            &spec.into(),
            sources.as_ptr(),
            sources.len(),
            core::ptr::null(),
            0,
            core::ptr::null(),
            0,
        )
    };
    if res.err != 0 {
        return Err(other_err("object create failed"));
    }
    Ok(res.val.into())
}

impl MmapInner {
    fn new(id: ObjID, map_flags: MapFlags, len: usize, off: usize) -> io::Result<MmapInner> {
        check_range(off, len)?;
        let handle = twizzler_rt_abi::object::twz_rt_map_object(id, map_flags)
            .map_err(|_| other_err("mmap failed"))?;
        Ok(Self { handle, len, off })
    }

    fn new_file(f: &File, map_flags: MapFlags, len: usize, off: u64) -> io::Result<MmapInner> {
        MmapInner::new(f.metadata()?.st_objid().into(), map_flags, len, off as usize)
    }

    /// Snapshot [off, off+len) of `f` into a fresh volatile object (kernel-level COW copy) and
    /// map that, so writes never reach the file: MAP_PRIVATE semantics.
    fn new_copy(f: &File, map_flags: MapFlags, len: usize, off: u64) -> io::Result<MmapInner> {
        let off = off as usize;
        check_range(off, len)?;
        let src_id: ObjID = f.metadata()?.st_objid().into();
        let copy_id = create_object(&[ObjectSource::new_copy(
            src_id,
            (NULLPAGE_SIZE + off) as u64,
            (NULLPAGE_SIZE + off) as u64,
            len,
        )])?;
        MmapInner::new(copy_id, map_flags, len, off)
    }

    pub fn map(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new_file(f, MapFlags::READ, len, off)
    }

    pub fn map_exec(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new_file(f, MapFlags::READ | MapFlags::EXEC, len, off)
    }

    pub fn map_mut(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        // Note: mmap never extends the file; writes land in-place within the existing length,
        // so no MEXT_SIZED maintenance is needed here.
        MmapInner::new_file(f, MapFlags::READ | MapFlags::WRITE, len, off)
    }

    pub fn map_copy(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new_copy(f, MapFlags::READ | MapFlags::WRITE, len, off)
    }

    pub fn map_copy_read_only(len: usize, f: &File, off: u64, _: bool) -> io::Result<MmapInner> {
        MmapInner::new_copy(f, MapFlags::READ, len, off)
    }

    pub fn map_anon(len: usize, _stack: bool, _populate: bool, _huge: Option<u8>) -> io::Result<MmapInner> {
        check_range(0, len)?;
        // Object pages are zero-filled on demand; no sources needed.
        let id = create_object(&[])?;
        MmapInner::new(id, MapFlags::READ | MapFlags::WRITE, len, 0)
    }

    fn sync(&self) -> io::Result<()> {
        if !self.handle.map_flags().contains(MapFlags::WRITE) {
            return Ok(());
        }
        // Kicks dirty pages of this mapping to the pager (same pattern as MutObject::sync);
        // a no-op for non-pager-backed (anon/copy) objects.
        let mut release = AtomicU64::new(0);
        let mut info = sync_info {
            release_ptr: addr_of_mut!(release).cast(),
            release_compare: 0,
            release_set: 1,
            durable_ptr: core::ptr::null_mut(),
            flags: SYNC_FLAG_DURABLE | SYNC_FLAG_ASYNC_DURABLE,
            __resv: 0,
        };
        self.handle
            .cmd(ObjectCmd::Sync, addr_of_mut!(info))
            .map_err(|_| other_err("sync failed"))
    }

    pub fn flush(&self, _off: usize, _len: usize) -> io::Result<()> {
        self.sync()
    }

    pub fn flush_async(&self, _off: usize, _len: usize) -> io::Result<()> {
        self.sync()
    }

    fn remap(&mut self, map_flags: MapFlags) -> io::Result<()> {
        // The mapping address changes, but memmap2's make_* conversions consume self, so no
        // borrows of the old mapping can outlive this.
        let handle = twizzler_rt_abi::object::twz_rt_map_object(self.handle.id(), map_flags)
            .map_err(|_| other_err("remap failed"))?;
        self.handle = handle;
        Ok(())
    }

    pub fn make_read_only(&mut self) -> io::Result<()> {
        self.remap(MapFlags::READ)
    }

    pub fn make_exec(&mut self) -> io::Result<()> {
        self.remap(MapFlags::READ | MapFlags::EXEC)
    }

    pub fn make_mut(&mut self) -> io::Result<()> {
        self.remap(MapFlags::READ | MapFlags::WRITE)
    }

    #[inline]
    pub fn ptr(&self) -> *const u8 {
        unsafe { self.handle.start().add(NULLPAGE_SIZE + self.off) }
    }

    #[inline]
    pub fn mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.handle.start().add(NULLPAGE_SIZE + self.off) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

pub fn file_len(file: &File) -> io::Result<u64> {
    Ok(file.metadata()?.len())
}
