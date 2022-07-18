// 字符读写相关的系统调用

// 打印至缓冲区
const FD_STDOUT: usize = 1;

// YOUR JOB: 修改 sys_write 使之通过测试
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // 从指针和长度转化为str
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            // 使用print!打印
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
