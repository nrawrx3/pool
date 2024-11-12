#![feature(allocator_api)]

use std::alloc::{Allocator, Layout, AllocError};
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::Mutex;

struct Block {
    next: Option<NonNull<Block>>
}

pub struct PoolAllocator {
    blocks: UnsafeCell<Option<NonNull<Block>>>,
    pool: UnsafeCell<NonNull<[u8]>>,
    lock: Mutex<()>,
    pool_size: usize,
    block_size: usize,
}

unsafe impl Send for PoolAllocator {}
unsafe impl Sync for PoolAllocator {}

impl PoolAllocator {
    pub fn new(block_size: usize, num_blocks: usize) -> Self {
        let pool_size = block_size * num_blocks;
        let layout = Layout::from_size_align(pool_size, std::mem::align_of::<Block>())
            .expect("Invalid layout parameters");

        let pool = unsafe {
            let ptr = std::alloc::alloc(layout);
            NonNull::slice_from_raw_parts(
                NonNull::new(ptr).unwrap(),
                pool_size
            )
        };

        Self {
            blocks: UnsafeCell::new(None),
            pool: UnsafeCell::new(pool),
            lock: Mutex::new(()),
            pool_size,
            block_size,
        }
    }

    fn init(&self) {
        let pool_ptr = unsafe { (*self.pool.get()).as_ptr() } as *mut u8;
        let mut prev_block = None;
        
        for i in (0..self.pool_size).step_by(self.block_size).rev() {
            let block = unsafe { &mut *(pool_ptr.add(i) as *mut Block) };
            block.next = prev_block;
            prev_block = Some(NonNull::new(block as *mut Block).unwrap());
        }
        
        unsafe {
            *self.blocks.get() = prev_block;
        }
    }
}

unsafe impl Allocator for PoolAllocator {
   fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() > self.block_size || layout.align() > self.block_size {
            // Fallback to system allocator for large allocations
            unsafe {
                let ptr = std::alloc::alloc(layout);
                NonNull::new(ptr)
                    .map(|p| NonNull::slice_from_raw_parts(p, layout.size()))
                    .ok_or(AllocError)
            }
        } else {
            let _guard = self.lock.lock().unwrap();
            
            unsafe {
                if (*self.blocks.get()).is_none() {
                    self.init();
                }

                if let Some(block) = (*self.blocks.get()) {
                    let next_block = (*block.as_ptr()).next;
                    *self.blocks.get() = next_block;
                    Ok(NonNull::slice_from_raw_parts(
                        NonNull::new_unchecked(block.as_ptr() as *mut u8),
                        layout.size()
                    ))
                } else {
                    // No blocks available, fallback to system allocator
                    let ptr = std::alloc::alloc(layout);
                    NonNull::new(ptr)
                        .map(|p| NonNull::slice_from_raw_parts(p, layout.size()))
                        .ok_or(AllocError)
                }
            }
        }
    } 

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() > self.block_size || layout.align() > self.block_size {
            // Was allocated with system allocator
            std::alloc::dealloc(ptr.as_ptr(), layout);
        } else {
            let _guard = self.lock.lock().unwrap();
            
            let block = &mut *(ptr.as_ptr() as *mut Block);
            block.next = *self.blocks.get();
            *self.blocks.get() = Some(NonNull::new_unchecked(block));
        }
    }
}

impl Drop for PoolAllocator {
    fn drop(&mut self) {
        unsafe {
            let pool = *self.pool.get();
            let layout = Layout::from_size_align(self.pool_size, std::mem::align_of::<Block>())
                .expect("Invalid layout parameters");
            std::alloc::dealloc(pool.as_ptr() as *mut u8, layout);
        }
    }
}