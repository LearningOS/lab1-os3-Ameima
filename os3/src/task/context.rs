//! 定义可以在汇编switch.S和Rust之间架起桥梁的任务上下文

#[derive(Copy, Clone)]
#[repr(C)] 
// 按照发生任务切换时保存现场的压栈顺序构造的结构体
pub struct TaskContext {
    ra: usize, // 返回地址
    sp: usize, // 栈顶
    s: [usize; 12], // s0~s11 寄存器
}

impl TaskContext {
    // 初始化函数，任务上下文全填零，创建任务表的时候用
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
