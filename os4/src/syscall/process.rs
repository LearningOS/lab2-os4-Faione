//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{translated_refmut, MapPermission, VirtAddr};
use crate::task::{
    current_task_info, current_user_token, exit_current_and_run_next, memeory_unmap, memory_map,
    suspend_current_and_run_next, TaskStatus,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();

    let token = current_user_token();
    *translated_refmut(token, ts) = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if (port & !0x7) != 0 || (port & 0x7) == 0 {
        return -1;
    }

    let len = ((len - 1) / 4096 + 1) * 4096;

    let start_va: VirtAddr = start.into();
    if start_va.page_offset() != 0 {
        return -1;
    }

    let end_va: VirtAddr = (start + len - 1).into();

    let mut map_perm = MapPermission::U;

    if port & 0x1 == 1 {
        map_perm |= MapPermission::R;
    };

    if (port >> 1) & 0x1 == 1 {
        map_perm |= MapPermission::W;
    };

    if (port >> 2) & 0x1 == 1 {
        map_perm |= MapPermission::X;
    };

    if let Err(err) = memory_map(start_va, end_va, map_perm) {
        error!(" sys_mmap err: {}", err);
        return -1;
    }

    0
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    let len = ((len - 1) / 4096 + 1) * 4096;

    let start_va: VirtAddr = start.into();
    if start_va.page_offset() != 0 {
        return -1;
    }

    let end_va: VirtAddr = (start + len - 1).into();

    if let Err(err) = memeory_unmap(start_va, end_va) {
        error!("sys_mmap err: {}", err);
        return -1;
    }

    0
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let token = current_user_token();
    *translated_refmut(token, ti) = current_task_info();
    0
}
