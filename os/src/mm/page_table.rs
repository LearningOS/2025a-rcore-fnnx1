//! Implementation of [`PageTableEntry`] and [`PageTable`].

use super::{frame_alloc, FrameTracker, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

// 一位一标志
bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        /// Valid
        const V = 1 << 0;
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// eXecutable
        const X = 1 << 3;
        /// User
        const U = 1 << 4;
        /// Global
        const G = 1 << 5;
        /// Accessed
        const A = 1 << 6;
        /// Dirty
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure PTE，页表项
pub struct PageTableEntry {
    /// bits of page table entry
    pub bits: usize,
}

impl PageTableEntry {
    /// Create a new page table entry
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    /// Create an empty page table entry
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    /// Get the physical page number from the page table entry 取出下一级页表的物理页号
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)/*这样相当于取[43：0]*/).into()
    }
    /// Get the flags from the page table entry
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    /// The page pointered by page table entry is valid?
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is readable?
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is writable?
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is executable?
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// page table structure 整个页表
pub struct PageTable {
    root_ppn: PhysPageNum,//根页表，也就是一级页表的物理页号
    frames: Vec<FrameTracker>,//在这个页表中的页表项，包括自身
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    /// Create a new page table
    /// 分配一个新物理页帧作为页表
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    /// 使用satp的值创建页表（根页表）
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    /// Find PageTableEntry by VirtPageNum, 
    /// create a frame for a 4KB page table if not exist
    /// 查找虚拟页号对应的页表项（最终映射到的物理页号）
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        //从根页表开始遍历三级页表
        for (i, idx) in idxs.iter().enumerate() {
            //从当前页表中取出索引位置的页表项
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);//当前是第三级页表，直接返回页表项（内容实际上是最终映射到的物理内存地址）
                break;
            }
            if !pte.is_valid() {//当前页表项无效，说明这个索引位置对应的下一级页表不存在
                //分配一个物理页帧来创建下一级页表
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();//访问下一级页表
        }
        result
    }
    /// Find PageTableEntry by VirtPageNum
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    /// set the map between virtual page number and physical page number
    /// 将一个虚拟页号映射到一个物理页号
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    /// remove the map between virtual page number and physical page number
    /// 解除虚拟页号到其指向物理页号的映射
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();//将三级页表项设置全0
    }
    /// get the page table entry from the virtual page number
    /// 其实就是把find_pte包装了一下
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    /// get the token from the page table
    /// token即satp寄存器的值
    pub fn token(&self) -> usize {
        //把根页表的物理页号放到低44位，模式设置为sv39，其他位置0
        8usize << 60/*8表示sv39*/ | self.root_ppn.0
    }
}

/// Translate&Copy a ptr[u8] array with LENGTH len to a mutable u8 Vec through page table
/// 通过页表将 ptr（虚拟地址）开始，长len bytes（虚拟地址差值）的数组 翻译为一个Vec（按页划分）
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    //根据satp值创建一个临时页表结构体
    //（个人理解：只要satp的值对应，实际上是访问现有页表，因为root_ppn是相同的）
    //但frames是新的，所以是临时
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;//起始地址（虚拟）作为usize
    let end = start + len;//不含end
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);//把当作虚拟地址使用
        let mut vpn = start_va.floor();//虚拟地址到虚拟页号
        //虚拟页号到物理页号
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();//+1，下一虚拟页
        let mut end_va: VirtAddr = vpn.into();//下一虚拟页号对应的虚拟地址
        end_va = end_va.min(VirtAddr::from(end));//end_va不能超过end
        if end_va.page_offset() == 0 {
            //说明前两步中end_va <= end，也就是说end不在当前页内
            //否则前两步end_va > end，这里end_va就是end，这说明end是与start同一页的
            //end.offset为0，那end等于start
            //根本不会进入本轮循环（符合“不包含end”的要求）
            //也就是说如果page_offset为0，说明当前页是完整使用的
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
            //start_va.page_offset()仅在第一轮循环中可能不为0，对应完整的ptr中的offset
        } else {
            //针对end在当前页内的情况
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
            //rust数组索引前闭后开，所以不含end
        }
        start = end_va.into();
    }
    v
}
