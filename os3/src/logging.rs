// 使用了外部库https://docs.rs/log/latest/log/index.html
use log::{self, Level, LevelFilter, Log, Metadata, Record};

// 自定义一个精简的Logger
struct SimpleLogger;

// 为其实现Log特征
impl Log for SimpleLogger {
    // 确定哪些需要被记录
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // 这里设置的是完全记录
        true
    }
    // 如何输出或存储记录，这里使用了println!
    fn log(&self, record: &Record) {
        // 排除未启用的
        if !self.enabled(record.metadata()) {
            return;
        }
        // 设置颜色
        let color = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 34,  // Blue
            Level::Debug => 32, // Green
            Level::Trace => 90, // BrightBlack
        };
        // 使用println!打印出来
        println!(
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    // 刷新缓冲
    fn flush(&self) {}
}

// 初始化logger
pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    // 根据运行程序时设置的环境变量来选择打印最大级别
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}
