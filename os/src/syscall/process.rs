//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, PhysAddr, VirtAddr},
    task::{
        change_program_brk, check_vpn_rw, create_new_map_area, current_user_token,
        exit_current_and_run_next, get_current_syscall_count, map_vpns_to_ppns,
        suspend_current_and_run_next, unmap_vpns, vaddr_to_paddr,
    },
    timer::get_time_us,
};
use core::ptr::{read_volatile, write_volatile};
use core::{mem::size_of, ptr::copy_nonoverlapping};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us: usize = get_time_us();
    let mut dst =
        translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    //取得 time_val 的字节表示
    let time_val_bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(
            (&time_val as *const TimeVal) as *const u8,
            size_of::<TimeVal>(),
        )
    };

    let mut offset = 0;
    for buf in dst.iter_mut() {
        let copy_len = buf.len().min(time_val_bytes.len() - offset);
        unsafe {
            copy_nonoverlapping(
                time_val_bytes.as_ptr().add(offset),
                buf.as_mut_ptr(),
                copy_len,
            );
        }
        offset += copy_len;
        if offset >= time_val_bytes.len() {
            break;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            if id > 0x0000003fffffffff {
                return -1;
            }
            let vpn = VirtAddr::from(id).floor();
            if !check_vpn_rw(vpn, 0) {
                return -1;
            }
            let p_addr: PhysAddr = vaddr_to_paddr(current_user_token(), id);
            unsafe { read_volatile(p_addr.0 as *const u8) as isize }
        }
        1 => {
            if id > 0x0000003fffffffff {
                return -1;
            }
            let vpn = VirtAddr::from(id).floor();
            if !check_vpn_rw(vpn, 1) {
                return -1;
            }
            let p_addr: PhysAddr = vaddr_to_paddr(current_user_token(), id);
            unsafe { write_volatile(p_addr.0 as *mut u8, data as u8) };
            0
        }
        2 => get_current_syscall_count(id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");

    if start % PAGE_SIZE != 0 || prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    }

    let start_va: VirtAddr = start.into();
    let end_va: VirtAddr = (start + len).into();
    //[start_vpn，end_vpn)
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    if map_vpns_to_ppns(start_vpn, end_vpn) {
        return -1;
    }

    let mut entry: MapPermission = MapPermission::empty();
    if prot & 1 != 0 {
        entry.insert(MapPermission::R);
    }
    if prot & 2 != 0 {
        entry.insert(MapPermission::W);
    }
    if prot & 4 != 0 {
        entry.insert(MapPermission::X);
    }
    entry.insert(MapPermission::U);

    create_new_map_area(start_va, end_va, entry);

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let start_va: VirtAddr = start.into();
    let end_va: VirtAddr = (start + len).into();
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();

    unmap_vpns(start_vpn, end_vpn)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
