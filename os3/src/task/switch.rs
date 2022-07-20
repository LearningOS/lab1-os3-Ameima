// 包装switch.S，这样我们是通过rust调用来进行转换的，而不是直接跳转到__switch，这样rust会帮忙保存栈
// __switch是在trap的情况下调用的。
// 当一个应用 Trap 到 S 模式的操作系统内核中进行进一步处理（即进入了操作系统的 Trap 控制流）的时候，
// 其 Trap 控制流可以调用一个特殊的 __switch 函数。

// 嵌入switch.S
core::arch::global_asm!(include_str!("switch.S"));

// 使用TaskContext
use super::TaskContext;

extern "C" {
    // 根据riscv调用约定，这俩参数会被放在a0和a1，switch.S中对应取用即可
    // a0中current_task_cx_ptr是当前上下文结构体，a1中next_task_cx_ptr是下一个程序上下文结构体
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
