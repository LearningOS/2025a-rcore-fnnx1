.altmacro//定义宏,相当于批量生成相同格式的代码
.macro SAVE_GP n
    sd x\n, \n*8(sp)//这里的\n对应上面的n, sd xi, i*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2//2^2 = 4字节(32位)对齐

//注意:观察发现,只要栈不是空着的, 每一次栈操作结束后内核栈顶指针指向的内容都是有值的, 而非我早期以为的sp指向栈的下(低地址)一个空位. 栈为空时KS的栈顶指针指向栈的开端(最高地址), 似乎一直留空.
__alltraps:
    //笔记
    //交换相应寄存器对于 内核栈/用户栈 栈顶 的指向
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    addi sp, sp, -34*8//把sp下移一个context的长度
    # save general-purpose registers
    //注意到sp指向地址对应x0,恒为0.上部刚好剩余33*8bytes空间
    //sd是把寄存器写进内存地址
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later 为啥?
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    //t4是线程指针
    # save x5~x31
    .set n, 5
    .rept 27//从5开始重复27次, 保存x[5:31]到内核栈
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved(就在上面) on kernel stack
    //把这两个csr里面的东西存进t0和t1寄存器
    csrr t0, sstatus
    csrr t1, sepc
    //csrr t2, sscratch 我觉得可以移到这里来达成绝妙的对称
    //riscv没有直接把控制状态寄存器数据写进内存的指令.
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch//这时候sscreath指向的是用户栈顶(也就是trap前的sp),之所以不在上面就存进内存应该是为了和下面的restore统一
    sd t2, 2*8(sp)

    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp//sp是内核栈顶地址, 和下面的mv sp, a0是一组"对偶操作"
    call trap_handler

//上面的逆操作,弹出栈来实现从s到u
__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0//调用_restore时a0是内核栈顶指针, 这个地方是启动应用程序传参用的
    # 现在sp指向内核栈 now sp->kernel stack(after allocated), sscratch->user stack

    //从内存中读出CSR(控制状态寄存器)的值
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    //写进对应CSR
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    //读出普通寄存器的值
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8//sp上移一个context
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    //sp这时候指向用户栈了
    sret
