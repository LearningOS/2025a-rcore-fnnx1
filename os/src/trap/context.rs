use riscv::register::sstatus::{self, Sstatus, SPP};
/// Trap Context
#[repr(C)]
//trap后内核把对应寄存器存进TC,压入内核栈
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus      
    pub sstatus: Sstatus,//特权级
    /// CSR sepc
    pub sepc: usize,//trap处理完成后的返回地址(trap处理完成后默认会执行的下一条指令的地址)
}
//查阅资料:rust结构体是按顺序紧密排布的,也就是&TC = &x[0], &TC[32] = &Ss...

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;//按内存排布来说刚好对应x2, 也就是被Trap.S约束(不按那个规范来就不能复用_restore了)的用于存放用户栈栈顶的位置
    }
    /// init app context //复用_restore来启动一个app时, 对内核栈按需压入寄存器状态以做准备
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read(); // CSR sstatus //riscv crate提供的接口
        sstatus.set_spp(SPP::User); //previous privilege mode: user mode
        let mut cx = Self {
            x: [0; 32],
            //程序刚开始运行时x0-x31都应该啥都没有
            sstatus,
            //该是啥就是啥,相当于暂时不变(把值挪来挪去似乎是不必要的性能损耗(可以简化掉), 可能是考虑到为了复用_restore)
            sepc: entry, // entry point of app
            //这样_restore之后就会把pc指向entry
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
