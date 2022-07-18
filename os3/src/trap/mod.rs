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

/// timer interrupt enabled
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
            cx.sepc += 4;
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
