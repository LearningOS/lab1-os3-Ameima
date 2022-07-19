# lab1实验报告
## 简答作业
1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容 (运行 Rust 三个 bad 测例 (ch2b_bad_*.rs) ， 注意在编译时至少需要指定 LOG=ERROR 才能观察到内核的报错信息) ， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。  

    答：RustSBI version 0.2.2。报错如下：  
    ```
        [ERROR] [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x80400408, core dumped.  
        [ERROR] [kernel] IllegalInstruction in application, core dumped.  
        [ERROR] [kernel] IllegalInstruction in application, core dumped.  
    ```

2. 
