//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// trace syscall
const SYSCALL_TRACE: usize = 410;


mod fs;
mod process;

extern crate alloc;
use alloc::vec::Vec;

use fs::*;
use process::*;

use crate::sync::UPSafeCell;
use lazy_static::*;

struct History {
    syscall_id: usize,
    time: usize,
}
/// syscall记录
pub struct SyscallTracer {
    history: UPSafeCell<Vec<History>>,
}

impl SyscallTracer{
    ///添加一个记录
    pub fn add(&self, id: usize) -> (){
        for his in self.history.exclusive_access().iter_mut() {
            if id == his.syscall_id {
                his.time += 1;
                return;
            }
        };
        self.history.exclusive_access().push(
            History{
                syscall_id: id,
                time: 1
            }
        );
        ()
    }
    ///查询历史记录
    pub fn query(&self, id: usize) -> usize{
        for his in self.history.exclusive_access().iter() {
            if id == his.syscall_id {
                return his.time;
            }
        }
        return 0;
    }
}


lazy_static!{
    ///调用记录结构体
    pub static ref TRACER:SyscallTracer = SyscallTracer{
        history: unsafe{UPSafeCell::new(Vec::<History>::new())},
    };
}

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    TRACER.add(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
