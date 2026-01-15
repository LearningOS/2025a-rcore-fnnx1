#![deny(warnings)]
#![allow(missing_docs)]
use bitflags::*;
use crate::task::processor::PROCESSOR;

// mmap 权限标志
bitflags! {
    pub struct MMapProt: i32 {
        const PROT_NONE  = 0;
        const PROT_READ  = 1 << 0;
        const PROT_WRITE = 1 << 1;
        const PROT_EXEC  = 1 << 2;
    }
}

// mmap 映射类型标志
bitflags! {
    pub struct MMapFlags: i32 {
        const MAP_SHARED    = 1 << 0;
        const MAP_PRIVATE   = 1 << 1;
        const MAP_ANONYMOUS = 1 << 2;
        const MAP_FILE      = 1 << 3;
    }
}

/// 处理mmap系统调用
pub fn do_mmap(addr: usize, length: usize, prot: MMapProt, flags: MMapFlags, fd: i32, offset: usize) -> Result<usize, i32> {
    let task = PROCESSOR.exclusive_access().current().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.mmap(addr, length, prot, flags, fd, offset)
}