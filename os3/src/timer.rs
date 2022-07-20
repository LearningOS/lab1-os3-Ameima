// 由于软件（特别是操作系统）需要一种计时机制，RISC-V 架构要求处理器要有一个内置时钟，其频率一般低于 CPU 主频。
// 此外，还有一个计数器用来统计处理器自上电以来经过了多少个内置时钟的时钟周期，该计数器保存在一个 64 位的 CSR mtime 中。
// 另外一个 64 位的 CSR mtimecmp 的作用是：一旦计数器 mtime 的值超过了 mtimecmp，就会触发一次时钟中断。
// 这使得我们可以方便的通过设置 mtimecmp 的值来决定下一次时钟中断何时触发。
// 它们都是 M 特权级的 CSR ，而我们的内核处在 S 特权级，是不被允许直接访问它们的。
// 好在运行在 M 特权级的 SEE （这里是RustSBI）已经预留了相应的接口，在sbi.rs中的set_timer封装。
use crate::config::CLOCK_FREQ; // 预先获取到的各平台不同的时钟频率，单位为赫兹，也就是一秒钟之内计数器的增量。
use crate::sbi::set_timer;
use riscv::register::time;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}

// 它首先读取当前 mtime 的值，然后计算出 10ms 之内计数器的增量，再将 mtimecmp 设置为二者的和。
// 这样，10ms 之后一个 S 特权级时钟中断就会被触发。反复调用则会实现按时中断。
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
