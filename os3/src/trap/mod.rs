// 使用上下文模块
mod context;

use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

// 内联trap的入口和出口，用于切换栈和保存寄存器
core::arch::global_asm!(include_str!("trap.S"));

// stvec是存储trap处理函数地址的寄存器
// 这里初始化就是把上面内联进来的处理函数入口设定进stvec里
pub fn init() {
    // 引入符号
    extern "C" {
        fn __alltraps();
    }
    // 设定
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

// 启用时间中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
// trap.S处理完以后会跳转至这里
// 
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {

    let scause = scause::read(); // 获取陷入原因，这俩都不是在cx上下文中的
    let stval = stval::read(); // 获取额外数据

    // 根据陷入原因进行分发处理
    match scause.cause() {
        // 请求ecall服务
        Trap::Exception(Exception::UserEnvCall) => {

            // 我们首先修改保存在内核栈上的 Trap 上下文里面 sepc，让其增加 4。
            // 这是因为我们知道这是一个由 ecall 指令触发的系统调用，
            // 在进入 Trap 的时候，硬件会将 sepc 设置为这条 ecall 指令所在的地址（因为它是进入 Trap 之前最后一条执行的指令）。
            // 而在 Trap 返回之后，我们希望应用程序控制流从 ecall 的下一条指令开始执行。
            // 因此我们只需修改 Trap 上下文里面的 sepc，让它增加 ecall 指令的码长，也即 4 字节。
            cx.sepc += 4;

            // 我们从 Trap 上下文取出作为 syscall ID 的 a7 和系统调用的三个参数 a0~a2 传给 syscall 函数并获取返回值。
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;

        }
        // 访问无权限访问的地址
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            error!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        // 使用非法指令
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next();
        }
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        // 未知陷入
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}

pub use context::TrapContext;
