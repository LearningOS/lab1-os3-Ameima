# 在 os/src/trap/trap.S 中实现 Trap 上下文保存/恢复的汇编代码
# 分别用外部符号 __alltraps（保存） 和 __restore（恢复） 标记为函数
# 并通过 global_asm! 宏将 trap.S 这段汇编代码插入到trap模块中封装使用。


# 启用alternate macro mode，为了后面结合宏使用.rept
.altmacro

# 定义两个宏SAVE_GP（保存寄存器x\n到sp指示的栈）和LOAD_GP（从栈还原寄存器x\n）
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    # 定义段
    .section .text

    # 导出两个标签作为trap中的符号
    .globl __alltraps
    .globl __restore
    
    # 将 __alltraps 的地址 4 字节对齐，这是 RISC-V 特权级规范的要求
    .align 2

__alltraps:
    # csrrw 原型是 csrrw rd, csr, rs 可以将 CSR 当前的值读到通用寄存器 rd 中，然后将通用寄存器 rs 的值写入该 CSR 。
    # 因此这里起到的是交换 sscratch 和 sp 的效果。
    # 在这一行之前 sp 指向用户栈， sscratch 指向内核栈，现在 sp 指向内核栈， sscratch 指向用户栈。
    csrrw sp, sscratch, sp

    # 准备在内核栈上保存 Trap 上下文，于是预先分配34*8字节的栈帧
    # riscv是没有push和pop的，sp栈顶，fp栈底
    # riscv是满减栈：sp指针一开始指向有内容的地址，进栈先向下移动指针后再将内容存进去，出栈先移动数据再向上移动指针。
    addi sp, sp, -34*8

    # sd保存双字，下面两条分别保存了x1和x3
    sd x1, 1*8(sp)
    # 跳过x2，因为2就是sp，一会儿要从交换后的sscratch取出之前的用户栈sp
    sd x3, 3*8(sp)
    # 跳过x4，线程指针，用不到

    # 从x5开始循环，保存寄存器
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # 从低地址到高地址分别按顺序放置 x0~x31这些通用寄存器，最后是 sstatus 和 sepc 。
    # 指令csrr的功能就是将 CSR 的值读到寄存器rd中。这里我们不用担心 t0 和 t1 被覆盖，因为它们刚刚已经被保存了。
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)


    # 从sscratch中读取用户堆栈地址并将其保存到内核堆栈中
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # 让寄存器 a0 指向内核栈的栈指针也就是我们刚刚保存的 Trap 上下文的地址
    # 这是由于我们接下来要调用 trap_handler 进行 Trap 处理，它的第一个参数 cx 由调用规范要从 a0 中获取。
    # trap_handler 需要 Trap 上下文的原因在于：它需要知道其中某些寄存器的值
    # 比如在系统调用的时候应用程序传过来的 syscall ID 和对应参数。
    # 我们不能直接使用这些寄存器现在的值，因为它们可能已经被修改了，因此要去内核栈上找已经被保存下来的值。
    mv a0, sp
    call trap_handler

__restore:

    # 先恢复 CSR 再恢复通用寄存器，这样我们使用的三个临时寄存器才能被正确恢复。

    # 恢复 CSR 
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2


    # 恢复通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr


    # 弹栈
    addi sp, sp, 34*8

    # 切换内核栈到用户栈，退场
    csrrw sp, sscratch, sp
    sret
