use riscv::register::sstatus::{self, Sstatus, SPP};
/// Trap Context
#[repr(C)]
//trap后内核把对应寄存器存进TC,压入内核栈
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus      
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,
}
//查阅资料:rust结构体是按顺序紧密排布的,也就是&TC = &x[0], &TC[32] = &Ss...

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;//按内存排布来说刚好对应x2, 也就是被Trap.S约束(不按那个规范来就不能复用_restore了)的用于存放用户栈栈顶的位置
    }
    /// init app context //这样之后就可以复用_restore来启动一个app了
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read(); // CSR sstatus
        sstatus.set_spp(SPP::User); //previous privilege mode: user mode
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // entry point of app
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}
