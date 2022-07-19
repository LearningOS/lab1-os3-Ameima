// 任务管理使用的结构，保存每个任务的当前状态信息

use super::TaskContext;

#[derive(Copy, Clone)]
// 每个任务的信息
pub struct TaskControlBlock {
    pub task_status: TaskStatus, // 任务状态
    pub task_cx: TaskContext, //任务上下文结构体
    // LAB1: Add whatever you need about the Task.
}

#[derive(Copy, Clone, PartialEq)]
// 任务的四种状态
pub enum TaskStatus {
    UnInit, // 未启动
    Ready, //挂起
    Running, // 正运行
    Exited, // 已结束
}
