# 实现任务切换

# 允许rept
.altmacro

# 保存和恢复s寄存器的宏
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm

    # 设定在.text段
    .section .text

    # 传递符号
    .globl __switch

__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # ) 两个参数分别在a0和a1

    # a0是目前的任务的上下文结构体
    # a1是下一个任务的上下文结构体

    # 保存sp到a0里
    sd sp, 8(a0)

    # 保存ra到a0里
    sd ra, 0(a0)

    # 保存s0~s11到a0里
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr

    # 从a1里恢复ra寄存器
    ld ra, 0(a1)

    # 从a1里恢复s0~s11寄存器
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr

    # 从a1里恢复sp寄存器
    ld sp, 8(a1)

    # 因为是当作rust函数来调用的，不是直接跳转，所以有返回
    ret

