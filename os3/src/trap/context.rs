//! 陷入上下文的实现 

use riscv::register::sstatus::{self, Sstatus, SPP};

// 定义TrapContext，为了给trap_handler用，通过Trap.S手动完成调用约定，构成了汇编与Rust之间的桥梁
#[repr(C)] // C布局，这样就和我们在Trap.S中入栈的顺序完美契合了，当a0中放入sp地址时，就可以将手动入栈的那些作为trap_handler参数
pub struct TrapContext {
    pub x: [usize; 32], // 32个通用寄存器
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {

    // 设定结构体中的x2（sp）
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    // 用于在初始化时在该应用的栈构造应用的Trap上下文
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User); // 将 sstatus 寄存器的 SPP 字段设置为 User 。
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // 修改其中的 sepc 寄存器为应用程序入口点 entry
        };
        cx.set_sp(sp); // sp 寄存器为我们设定的用户栈指针
        cx // 返回构造好的上下文
    }
}
