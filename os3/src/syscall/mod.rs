// 系统调用实现

// 调用种类对应的ID
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_TASK_INFO: usize = 410;

mod fs; // 字符读写相关的系统调用
mod process; //文件读写相关的系统调用 

use fs::*;
use process::*;

// syscall 函数并不会实际处理系统调用，而只是根据 syscall ID 分发到具体的处理函数
// trap_handler从上下文中取出a7作为syscall_id，取出a0~a2作为参数调用它
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // LAB1: You may need to update syscall info here.
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
