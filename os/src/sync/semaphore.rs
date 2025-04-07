//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
use alloc::vec::Vec;
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    ///
    pub count: isize,
    ///
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
    ///
    pub allocated_queue: Vec<usize>
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                    allocated_queue:Vec::new()
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        
        let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        for (i,inner_tid) in inner.allocated_queue.iter().enumerate() {
            if *inner_tid == tid {
                inner.allocated_queue.remove(i);
                break;
            }
        }
        
        
        inner.count += 1;
        
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                inner.allocated_queue.push(current_task().unwrap().get_tid());
                wakeup_task(task);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        
        
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }else{
            let tid = current_task().unwrap().get_tid();
            inner.allocated_queue.push(tid);
        }
    }

    ///
    pub fn count(&self)->i32{
        let inner = self.inner.exclusive_access();
        if inner.count<0{0}else{inner.count as i32}
    }




}