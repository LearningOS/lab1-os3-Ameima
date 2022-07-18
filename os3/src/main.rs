// 去除std与main
#![no_std]
#![no_main]

// 加上 #![feature(panic_info_message)] 才能通过 PanicInfo::message 获取报错信息
#![feature(panic_info_message)]

// 允许编写动态内存分配失败时的处理函数
#![feature(alloc_error_handler)]

// 使用log库
#[macro_use]
extern crate log;

// 使用alloc库以支持buddy_system_allocator
extern crate alloc;

#[macro_use]
mod console;
mod config;
mod heap_alloc;
mod lang_items;
mod loader;
mod logging;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

// 内联入口点汇编
core::arch::global_asm!(include_str!("entry.asm"));

// 内联各个用户态程序
core::arch::global_asm!(include_str!("link_app.S"));

// 清零bss段
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[no_mangle] // 保留符号，作为rust部分入口函数
pub fn rust_main() -> ! {
    clear_bss(); // 清零bss
    logging::init(); // 初始化logger
    println!("[kernel] Hello, world!");
    heap_alloc::init_heap(); // 初始化堆？？？为什么现在就有堆了
    trap::init(); // 初始化trap，处理所有的U陷入S
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}
