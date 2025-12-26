//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.
mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, translated_refmut, translated_str, PageTableEntry};
use page_table::{PTEFlags, PageTable};
/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();//这个"初始化"除了设置了一个可用物理页帧范围实际上啥都没做
    KERNEL_SPACE.exclusive_access().activate();//这里启用内核空间(虚拟地址)时应该会把8000_0000~8800_0000恒等映射进页表.
}
