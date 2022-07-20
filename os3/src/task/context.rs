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

    // 用于构造任务切换的上下文，接受的参数是目前的内核栈栈顶，返回的是任务切换的上下文结构体
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize, // 返回地址寄存器设置为从Trap上下文还原的函数
            // 这样任务切换上下文首先被还原，还原完后直接跳转到还原Trap上下文的地方，继续还原Trap上下文
            // 然后就可以开始运行程序自己的部分了，属于是套了两层娃
            sp: kstack_ptr, // Trap上下文的栈顶，也就是套进任务切换上下文之前的sp
            s: [0; 12], // 毕竟还没运行，没有用到s寄存器，所以全部填零即可
        }
    }
}
