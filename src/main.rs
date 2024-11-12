#![feature(allocator_api)]

use pool::PoolAllocator;

fn main() {
    let pool = PoolAllocator::new(64, 16);
    
    // Now we can create Vec with our allocator
    let mut vec = Vec::new_in(&pool);
    vec.push(42);
    
    println!("Vec with custom allocator: {:?}", vec);

    let mut vec2 = Vec::new_in(&pool);
    vec2.push(43);
}