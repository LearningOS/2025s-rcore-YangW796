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
    pub allocated_queue: Vec<usize>,
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
                    allocated_queue: Vec::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;
        let tid = current_task().unwrap().get_tid();
        for (i, inner_tid) in inner.allocated_queue.iter().enumerate() {
            if *inner_tid == tid {
                inner.allocated_queue.remove(i);
                break;
            }
        }
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                inner
                    .allocated_queue
                    .push(task.get_tid());
                wakeup_task(task);
            }
        } 
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let tid=current_task().unwrap().get_tid();
        println!("kernel: Semaphore::down,{}1",tid);
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        println!("kernel: Semaphore::down,{}2",tid);
        if inner.count < 0 {
            println!("kernel: Semaphore::down,{}3",tid);
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }// } else {
        //     println!("kernel: Semaphore::down,{}4",tid);
        //     let current_task = current_task().unwrap();
        //     let tid = current_task.get_tid();
        //     inner.allocated_queue.push(tid);
        //     drop(current_task);
        //     drop(inner);
        // }
    }

    ///
    pub fn count(&self) -> i32 {
        let inner = self.inner.exclusive_access();
        if inner.count < 0 {
            0
        } else {
            inner.count as i32
        }
    }

    ///
    pub fn allocated(&self) -> Option<Vec<usize>> {
        let inner = self.inner.exclusive_access();
        if inner.allocated_queue.len() > 0 {
            return Some(inner.allocated_queue.clone());
        } else {
            return None;
        }
    }

    ///
    pub fn need(&self) -> Option<Vec<usize>> {
        let inner = self.inner.exclusive_access();
        let n = inner.wait_queue.len();
        if n == 0 {
            return None;
        } else {
            let mut res = Vec::new();
            for i in 0..n {
                let task = &inner.wait_queue[i];
                res.push(task.get_tid());
            }
            return Some(res);
        }
    }
}
