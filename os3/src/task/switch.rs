// 包装switch.S

core::arch::global_asm!(include_str!("switch.S"));

use super::TaskContext;

extern "C" {
    // 根据riscv调用约定，这俩参数会被放在a0和a1，switch.S中对应取用即可
    // a0中current_task_cx_ptr是当前上下文结构体，a1中next_task_cx_ptr是下一个程序上下文结构体
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
