//伪操作
    .section .text.entry# (告诉链接器)下面的内容放到text的entry段(程序的开始)
    .globl _start//标签(对应一个链接后的地址)

_start://从这个标签开始放置一下内容到对应段
    la sp, boot_stack_top
    call rust_main #跳进了rust_main,下面的代码的执行?(伪操作,只在链接时起作用?)

    .section .bss.stack#下面的内容放进.bss.stack段
    .globl boot_stack_lower_bound

boot_stack_lower_bound:
    .space 4096 * 16//分配内存,从这里开始往上数4096*16 Bytes = 65536B留空给栈

    .globl boot_stack_top
boot_stack_top: