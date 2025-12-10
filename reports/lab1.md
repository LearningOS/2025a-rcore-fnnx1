荣誉准则
---

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    实验完成过程中，我未与他人进行相关部分的直接交流。但我曾经与网友进行过有关rust知识的讨论，内容未在实验中直接体现，所以省略。
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    我参考了`https://cloud.tencent.com/developer/article/2345493`学习有关初始化数组的代码。
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

我实现的功能描述
---

在已有代码的基础上，我通过修改`TaskManager`结构体，引入`SyscallTracer`结构体成员`tracer`（以及实现了有关方法），实现了在各个程序中分别对几种syscall的记录`add_record()`和查询`query_record()`。此外还通过unsafe+裸指针操作实现了内存读写。

简答题
---

1
程序被杀死了，继续加载下一个程序并运行。使用RustSBI version 0.3.0-alpha.4：

```bash
[kernel] Loading app_0
[kernel] PageFault in application, kernel killed it.
[kernel] Loading app_1
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] Loading app_2
[kernel] IllegalInstruction in application, kernel killed it.
```

2
    1.sp代表内核栈顶。a.从trap恢复；b.应用程序启动。
    2.特殊处理t0-2和csr，意义是把csr恢复为“原来的”（或者启动应用时需要的）值。
    3.x2是sp，传入参数的时候就加载了；x4是线程指针，用不到。
    4.sp用户栈顶，ss内核
    5.sret，riscv中csr会在这条指令之后自动修改为低一级权限。
    6.sp内核，ss用户
    7.ecall

---
    建议：讲解可以写得易读一点。
