/*
ld链接脚本（Linker Script）是控制程序内存布局的关键文件
https://blog.csdn.net/zxcvbnmjdjs/article/details/148927526
*/

OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

/*给不同段分配一个具体空间*/
SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;/*start*/
    .text : {
        *(.text.entry) /* <ObjectFile>(SectionName) */
        *(.text .text.*)
    }

    . = ALIGN(4K);/*向高地址对齐到4k字节*/
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)/*Small Read-Only Data*/
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}