mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::syscall::process::TaskInfo; // 新增
use crate::timer::get_time_us;
use lazy_static::*;
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

// 任务表，初始化后本体不变，使用UPSafeCell实现内部可变
pub struct TaskManager {
    num_app: usize, // 任务总数
    inner: UPSafeCell<TaskManagerInner>, // 可变部分
}

// 任务表可变部分
struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM], // 各个任务的信息
    current_task: usize, // 当前正在执行哪个任务
}

// lazy_static! 宏提供了全局变量的运行时初始化功能。
// 一般情况下，全局变量必须在编译期设置一个初始值，但是有些全局变量依赖于运行期间才能得到的数据作为初始值。
// 这导致这些全局变量需要在运行时发生变化，即需要重新设置初始值之后才能使用。
// 如果我们手动实现的话有诸多不便之处，比如需要把这种全局变量声明为 static mut 并衍生出很多 unsafe 代码 。
// 这种情况下我们可以使用 lazy_static! 宏来帮助我们解决这个问题。
lazy_static! {

    pub static ref TASK_MANAGER: TaskManager = {

        // 获取应用总数
        let num_app = get_num_app();

        // 构造任务表
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            // 新增
            task_syscall_times: [0; MAX_SYSCALL_NUM],
            task_first_running_time: None,
        }; MAX_APP_NUM];

        // 启动各个任务，初始化到挂起状态
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            // 如果应用是第一次被执行，那内核应该怎么办呢？
            // 类似构造 Trap 上下文的方法，内核需要在应用的任务控制块上构造一个用于第一次执行的任务上下文。
            // 我们是在创建 TaskManager 的全局实例 TASK_MANAGER 的时候来进行这个初始化的，就在下面这句。
            // init_app_cx是压Trap上下文进内核栈，返回新的栈顶
            // goto_restore是在压Trap上下文进内核栈的基础上，构建任务切换的上下文结构体，不压栈而是直接返回
            // 这俩上下文创造函数套一起，返回的是任务上下文结构体，刚好放进t的任务上下
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }

        // 封装成任务表返回
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    // CPU 第一次从内核态进入用户态。
    // 只需在内核栈上压入构造好的 Trap 上下文，然后 __restore 即可。
    fn run_first_task(&self) -> ! {
        // 获取任务表可变部分的一个独占借用（也就是可变借用）给inner
        let mut inner = self.inner.exclusive_access();
        // 从中取出第一个任务
        let task0 = &mut inner.tasks[0];
        // 状态设置为正在运行
        task0.task_status = TaskStatus::Running;

        // 新增：对初次调度时间则进行设置
        task0.task_first_running_time = Some(get_time_us());

        // 把第一个任务放进下一个要运行的任务中，以供一会儿__switch使用
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        // 手动清理掉临时变量，因为这个函数没到达底部前就切出去了而且永远不会回来，不能依靠编译器自己清除
        drop(inner);
        // 因为之前没有任务运行过，所以创建一个空任务上下文，从空任务上下文利用__switch切换到第一个任务
        let mut _unused = TaskContext::zero_init();
        // 发起切换，操作系统会以为Trap之前运行的是这个空任务，所以保存现场
        // 首先会把当前的现场全都压到这个空任务上下文中，之后从第一个任务上下文中还原现场
        // 第一个任务上下文中目前是我们构造的初始化上下文，s寄存器全是0，sp寄存器在我们构造的Trap上下文的栈顶上，ra是Trap上下文的恢复函数
        // 调用完__switch后，编译器自动跳到此时的ra，也就进一步开始了Trap上下文的恢复过程。构造的Trap上下文包含全部的寄存器。
        // 初始的Trap上下文中，sepc是应用本身的入口点，sstatus是U特权级，sp是用户栈底，其余是0。这样Trap恢复后就会换到对应应用的用户栈底，然后从头开始执行应用。
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    // 应用状态设置为挂起
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    // 应用状态设置为已结束
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    // 寻找下一个挂起的应用，从当前的ID顺延查找
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    // 切换到下一个任务
    fn run_next_task(&self) {
        // 先寻找还有没有挂起的任务
        if let Some(next) = self.find_next_task() {
            // 类似应用首次运行的过程，不过不用创造空任务了
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;

            // 新增：如果没有被调度过，则对初次调度时间则进行设置
            if inner.tasks[next].task_first_running_time == None {
                inner.tasks[next].task_first_running_time = Some(get_time_us());
            }

            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }

        // 没有挂起的任务了，全部运行完毕
        } else {
            panic!("All applications completed!");
        }
    }

    // LAB1: Try to implement your function to update or get task info!

    // 增加对应ID的系统调用计数
    fn update_syscall_times(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_syscall_times[syscall_id] += 1;
    }

    // 获取当前应用任务信息
    fn get_task_info(&self) -> TaskInfo {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let time = get_time_us() - inner.tasks[current].task_first_running_time;
        TaskInfo {
            status: inner.tasks[current].task_status,
            syscall_times: inner.tasks[current].task_syscall_times,
            time,
        }
    }
}

// 留给main函数调用的接口
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

// 下面的都是些封装，没有pub，因为是给更下面的那些封装用的

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

// 这些是给外界调用的接口

// 当前应用挂起，运行下一个应用
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

// 当前应用退出，运行下一个应用
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

// LAB1: Public functions implemented here provide interfaces.
// You may use TASK_MANAGER member functions to handle requests.

// 增加对应ID的系统调用计数
pub fn update_syscall_times(syscall_id: usize) {
    TASK_MANAGER.update_syscall_times(syscall_id);
}

// 获取当前应用任务信息
pub fn get_task_info() -> TaskInfo {
    TASK_MANAGER.get_task_info()
}