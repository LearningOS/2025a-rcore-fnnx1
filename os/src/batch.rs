//! batch subsystem

use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;
use lazy_static::*;

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    //栈的最高地址,但是是栈底?
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr //这样写栈的最高地址处应该是0,而cx_ptr指向的位置应该是有数据的
            = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],//参考下面的打印, 最后一个app也有一个结束地址,所以多留1
} 

impl AppManager {
    pub fn print_app_info(&self) {//序号和起始地址
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }
    
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed!");
            use crate::board::QEMUExit;
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
        println!("[kernel] Loading app_{}", app_id);
        // clear app area
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst 
            = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
        // Memory fence about fetching the instruction memory
        // It is guaranteed that a subsequent instruction fetch must
        // observes all previous writes to the instruction memory.
        // Therefore, fence.i must be executed after we have loaded
        // the code of the next app into the instruction memory.
        // See also: riscv non-priv spec chapter 3, 'Zifencei' extension.
        asm!("fence.i");
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}


//声明为全局静态变量, lazy...和UPSC保证多线程安全, 具体原理还不懂
lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();//in link_app.S
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();//读指针指向的值,参考link_app.S
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);//在linkapp.s中num_pp符号指向一个数组,
                //里面除了数量还有起始地址
            app_start[..=num_app].copy_from_slice(app_start_raw);//[0]对齐
            AppManager {
                num_app,//数量
                current_app: 0,//当前no,初始化为0
                app_start,//共numapp+1项有值
            }
        })
    };
}

/// init batch subsystem
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// run next app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app 
        = app_manager.get_current_app();//app序号
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);
    //以上操作是调整app_manager的状态

    // before this we have to drop local variables related to resources manually
    // and release the resources
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    //启动app本质上和trap的恢复是相同的过程, 于是这里复用_restore

    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize/*为了模拟"trap到s模式后的状态", 按Trap.S的约定把app开始运行时需要的寄存器值(主要是用户栈栈顶地址)写进内核栈顶, 调用_restore之后把这些东西读进寄存器, 程序就自然地启动了:)*/);
    }
    panic!("Unreachable in batch::run_current_app!");
}
