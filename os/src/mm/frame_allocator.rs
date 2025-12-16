//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use super::{PhysAddr, PhysPageNum};
use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

/// tracker for physical page frame allocation and deallocation
pub struct FrameTracker {
    /// physical page number
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    /// Create a new FrameTracker
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();//物理内存上长度4k的切片（首项指针+长度）
        for i in bytes_array {
            *i = 0;//清零
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

//drop在rust结果体中类似cpp类的析构函数，会自动调用
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);//析构时自动释放物理页帧
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}
/// an implementation for frame allocator 
/// 叫这个名字是因为它的分配方式符合栈，
/// 分配的不是栈帧
/// 个人翻译：“用栈实现的物理页帧分配器”
pub struct StackFrameAllocator {
    current: usize,//当前分配到的位置（页号，即地址单位4k）
    end: usize,//内存末尾
    recycled: Vec<usize>,//被分配过但又被释放的页帧号集合
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;//可用物理内存区间低地址
        self.end = r.0;//可用内存区间高地址
        // trace!("last {} Physical Frames.", self.end - self.current);
    }
}

//为“用栈实现的物理页帧分配器”实现帧分配器特征
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        //如果有回收的
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {//判断是否没有剩余空间
            None
        } else {//分配新空间
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check 检查指定物理页号是否被分配
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

//这个类型定义表明rcore的帧分配器实现是“用栈实现的物理页帧分配器”
type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();//end of kernel, 内核结束地址
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );//从内核结束开始，内存末尾结束
}

/// Allocate a physical page frame in FrameTracker style
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
        //最后这行等价于.map(|x| FrameTracker::new(x))
}

/// Deallocate a physical page frame with a given ppn
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
/// 请求分配5个页帧，释放（这时候应该能自动deallocate），再请求分配5个页帧，释放
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
