use my_allocator::*;

use std::alloc::{GlobalAlloc, Layout};

// NOTE: Print statements inside allocator can cause hangs. May also be because dealloc doesn't work yet.
//#[global_allocator]
static MY_ALLOX: Allox = Allox::new();
//#[global_allocator]
//static TEST_ALLOC: SimpleAlloc = SimpleAlloc::new();

fn with_normal_global() {
	let a = Allox::new();
	println!("Init:\n{a:?}");

	let l1 = Layout::from_size_align(128, 1).unwrap();

	let mut ptrs = Vec::new();

	ptrs.push(unsafe { a.alloc(l1) });
	println!("Allocated {l1:?} to {:?}", ptrs.last().unwrap());
	println!("{a:?}");

	ptrs.push(unsafe { a.alloc(l1) });
	println!("Allocated {l1:?} to {:?}", ptrs.last().unwrap());
	println!("{a:?}");

	for p in ptrs {
		unsafe {
			a.dealloc(p, l1);
		}

		println!("{a:?}");
	}
}

fn main() {
	//let v = vec![1; 32];
	with_normal_global();
}
