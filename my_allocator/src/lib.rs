//! Experimental memory allocator.
//!

use core::{
	alloc::{GlobalAlloc, Layout},
	sync::atomic::{AtomicUsize, AtomicPtr, Ordering},
};
use syscalls::{
	syscall,
	Sysno,
};
const MAP_ANONYMOUS: usize = 0x20;
const MAP_PRIVATE: usize = 0x2;
const PROT_READ: usize = 0x1;
const PROT_WRITE: usize = 0x2;

const ALLOX_POOL_SIZE: usize = 4;
const PTR_BITS: usize = core::mem::size_of::<usize>() * 8;

/** Custom allocator and notes.

# Desires
Min-max heap for available slots.

# Current chicken scratch
`viewers` is incremented any time a thread starts using a sector entry. After
 all operations it is decremented.

## Allocation
1. Round requested space up to multiple of slot size (or power of 2).
2. Iterate non-null array entries.
3. If sector has space available, claim the first available space.
4. Else continue through array.
5. If unsuccessful at end of array, allocate new sector and claim space.
6. Attempt to insert new sector to end of array, else return failure (null).
	a. If `num_sectors` is less than array length, increment.
	b. Write sector pointer to array.
7. Return success (allocated ptr).

Sectors in pool are available if `base` is non-null.

## Freeing
Repeatedly iterate array until sector containing pointer is found. Deallocate
 sector if necessary or desired.

Repeats only occur due to deallocation currently happening.

## Deallocation:
Make sure to think about any free/allocate operations presently executing, both
 before and after current position in array.

TODO: Rework given newfound experience. Generally just add detail to order of ops.

1. Set `viewers` from 0 to MAX/-1. If fail, stop.
2. Check if entire sector is free (`slots` == 0).
3. Null sector pointer in array.
4. Free allocation, but leave `base` pointer set.
5. If sector is end of array, decrement array length (CAS) and null the sector `base` pointer.
6. Else, move last pointer to local variable and swap with null (CAS).
	a. If swap fails (ptr value changed), or ptr is null, retry.
	b. If sector is now end of array, then do step 5.
7. Overwrite current slot array pointer with the moved one from 6. (Non-atomic?)
8. Decrement array length.
9. Write null pointer into the deallocated sectors `base` pointer. (Non-atomic?)
10. Done?

Note: Steps 7 and 8 should be interchangable, depending whether existing ops or
 new sector growth is prioritized (on a micro scale).

### Safety contract
1. All pointers in `sectors` shall be valid or null. (Currently should point into `sector_pool`).
2. Sector pool size is fixed, so any pointer to a sector within is valid.
	a. If dynamic pool size, recommend only growing pool to avoid complications
	    shrinking. Pool memory is likely relatively small compared to total.
3. A [Sector] whose pointer exists in `sectors` shall have a valid and
    allocated `base` pointer.

**/
#[derive(Debug)]
pub struct Allox {
	/// Allocated sector array.
	// Use of pointers necessary for atomic operations.
	sectors: [AtomicPtr<Sector>; ALLOX_POOL_SIZE],
	/// How many allocated sectors are there in array ("length").
	num_sectors: AtomicUsize,
	/// Allocation count for debugging/profiling
	total_allocs: AtomicUsize,
	/// Sectors available for use. Extra slot just in case lots of alloc activity.
	sector_pool: [Sector; ALLOX_POOL_SIZE + 1],
}
impl Allox {
	pub const fn new() -> Self {
		// Transmute necessary since atomics are not [Copy].
		// SAFETY: [AtomicPtr] guaranteed same size as `*mut Sector`.
		unsafe {
			Allox {
				sectors: core::mem::transmute([core::ptr::null_mut::<Sector>(); ALLOX_POOL_SIZE]),
				num_sectors: AtomicUsize::new(0),
				sector_pool: [Sector::NULL; ALLOX_POOL_SIZE + 1],
				total_allocs: AtomicUsize::new(0),
			}
		}
	}

	// Iterate sector pool and search for available/unallocated sector
	pub fn add_sector_for(&self, lay: Layout) -> Result<*mut u8, ()> {
		// Do initial check if there's room
		if self.num_sectors.load(Ordering::Relaxed) >= self.sectors.len() {
			return Err(())
		}

		// Find available sector
		let mut sec = None;
		for (_i,s) in self.sector_pool.iter().enumerate() {
			if s.base.load(Ordering::Acquire).is_null() {
				if s.viewers.compare_exchange(0, usize::MAX, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
					sec = Some(s);
					println!("Found unallocated sector at [{_i}]");
					break;
				}
			}
		}

		// If sector found, allocate.
		match sec {
			// Allocate sector large enough so that `num_bits` <= PTR_BITS.
			Some(s) if s.alloc().is_ok() => {
				// Compute number of chunks/bits needed and claim first slots.
				let num_bits = lay.size().div_ceil(Sector::CHUNK_SIZE);
				let r = (!0) >> (PTR_BITS - num_bits);
				s.slots.store(r, Ordering::Release); // Claim slots

				// Reset viewer count. Safe now that base is non-null.
				s.viewers.store(0, Ordering::Release);

				// Try to push sector to array
				let n = self.num_sectors.load(Ordering::Acquire);
				if n < self.sectors.len() {
					if self.num_sectors.compare_exchange(n, n+1, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
						// Store sector pointer
						self.sectors[n].store(s as *const _ as *mut _, Ordering::Release);

						return Ok(s.base.load(Ordering::Relaxed))
					}
				}

				Err(())
			},
			// If allocation fails, still reset viewer count.
			Some(s) => {
				s.viewers.store(0, Ordering::Release);
				Err(())
			},
			_ => Err(()),
		}
	}

	/// Iterate over valid [sectors] entries, calling `fun`, and returning the
	///  first [Some].
	///
	/// User function is called after a sector's `viewers` count has been
	///  incremented.
	///
	/// # Safety
	/// User function can modify the entire sector, or potentially invalidate
	///  the sector memory as a whole (since it can mutate the [Allox] object),
	///  so it is on the user to avoid invalidating the reference given to the
	///  function.
	///
	/// In short, the user must uphold the "Safety Contract" as specified in
	///  the [Allox] docs.
	unsafe fn iter_sectors<T, F: Fn(usize, &Sector) -> Option<T>>(&self, fun: F) -> Option<T> {
		// Iterate existing sectors
		for i in 0..self.num_sectors.load(Ordering::Relaxed) {
			let s = &self.sectors[i];
			// Load pointer and convert to [Sector] ref (if non-null).
			// SAFETY: See "Safety Contract" above.
			let p = s.load(Ordering::Relaxed);
			if let Some(sec) = unsafe { p.as_ref() } {
				// Check if not at max, then increment.
				let v = sec.viewers.load(Ordering::Acquire);
				if v < usize::MAX {
					// Increment viewer count before use.
					let r = sec.viewers.compare_exchange(v, v+1, Ordering::AcqRel, Ordering::Relaxed);

					// If viewers successfully incremented, call user fn.
					if let Ok(_) = r {
						if let Some(r) = fun(i, sec) {
							// Decrement viewer since usage is complete.
							sec.viewers.fetch_sub(1, Ordering::Release);
							return Some(r)
						}
					} else {
						// Decrement viewer since usage is complete.
						sec.viewers.fetch_sub(1, Ordering::Release);
					}
				}
			}
		}

		None
	}
}
unsafe impl GlobalAlloc for Allox {
	unsafe fn alloc(&self, lay: Layout) -> *mut u8 {
		// Try to allocate memory in existing sector
		let res = unsafe {
			self.iter_sectors(|i, sec| {
				println!("Searching in sector [{i}]..");

				// If request is successful, return ptr.
				match sec.request_mem(lay) {
					Ok(ptr) => Some(ptr),
					_ => None,
				}
			})
		};

		// If successfully allocated in existing sector, return ptr.
		if let Some(p) = res {
			self.total_allocs.fetch_add(1, Ordering::Relaxed);
			return p
		}

		// Else try to allocate new sector then assign memory there.
		match self.add_sector_for(lay) {
			Ok(p) => {
				self.total_allocs.fetch_add(1, Ordering::Relaxed);
				p
			},
			Err(_) => core::ptr::null_mut(),
		}
	}

	unsafe fn dealloc(&self, ptr: *mut u8, lay: Layout) {
		println!("Deallocating {ptr:?} with layout {lay:?}..");

		let _idx = self.iter_sectors(|i,sec| {
			match sec.release_mem(ptr, lay) {
				Ok(_) => Some(i),
				Err(_) => None,
			}
		});

		self.total_allocs.fetch_sub(1, Ordering::Relaxed);
	}
}

#[derive(Debug)]
pub struct Sector {
	/// How many threads are currently using this sector object. Yes usize is overkill.
	// usize::MAX or -1 indicates sector is being freed.
	viewers: AtomicUsize,
	/// Bitmap of slot status (1 - in use, 0 - Free).
	slots: AtomicUsize,
	/// First address in the allocated sector.
	base: AtomicPtr<u8>, // Use u8 ptr??
}
impl Sector {
	pub const NULL: Self = Self::null(); // Used for situations requiring [Copy].

	// Parameters of sectors (for fixed-size sectors).
	pub const ALLOC_SIZE: usize = 4096;
	pub const CHUNK_SIZE: usize = Self::ALLOC_SIZE / (8 * core::mem::size_of::<usize>());

	pub const fn null() -> Self {
		Sector {
			viewers: AtomicUsize::new(0),
			slots: AtomicUsize::new(0),
			base: AtomicPtr::new(core::ptr::null_mut()),
		}
	}

	/// Allocate memory for this sector, if not already allocated. Fails if
	///  already allocated.
	pub fn alloc(&self) -> Result<(), ()> {
		// Fail is sector is already allocated
		if !self.base.load(Ordering::Relaxed).is_null() {
			return Err(())
		}

		let p = unsafe { syscall!( Sysno::mmap,
			0,
			Self::ALLOC_SIZE,
			PROT_READ | PROT_WRITE,
			MAP_ANONYMOUS | MAP_PRIVATE,
			0,
			0
		)};

		// Convert return to a valid pointer.
		let p = p.map_or(core::ptr::null_mut(), |v| v as *mut u8);

		// Set pointer to self, or unmap on failure.
		match self.base.compare_exchange(core::ptr::null_mut(), p, Ordering::Release, Ordering::Relaxed) {
			Ok(_) => {
				//println!("Sector successfully allocated!");
				Ok(())
			},
			Err(_) => {
				_ = unsafe { syscall!(Sysno::munmap, p as usize, Self::ALLOC_SIZE) };
				Err(())
			},
		}
	}

	pub fn dealloc(&self) {
		_ = unsafe { syscall!(Sysno::munmap, self.base.load(Ordering::Relaxed) as usize, Self::ALLOC_SIZE) };
	}

	/// Attempt to claim a region of memory for `lay` and return start address.
	pub fn request_mem(&self, lay: Layout) -> Result<*mut u8, ()> {
		let num_bits = lay.size().div_ceil(Sector::CHUNK_SIZE);
		let mut r = (!0) >> (PTR_BITS - num_bits);
		let map = self.slots.load(Ordering::Acquire);
		println!("Checking against existing slots 0x{map:x}..");
		for i in 0..(PTR_BITS - num_bits) {
			if (map & r) == 0 {
				let res = self.slots.compare_exchange(map, map | r, Ordering::AcqRel, Ordering::Relaxed);
				if res.is_ok() {
					// SAFETY: `i < PTR_BITS` so the chunk will always start inside base allocation range.
					unsafe {
						return Ok(self.base.load(Ordering::Relaxed).add(Sector::CHUNK_SIZE * i))
					}
				}
			}

			r <<= 1; // Shift r to align with map
		}

		Err(())
	}

	/// Release the memory for the given address and layout, if it is within this sector.
	pub fn release_mem(&self, addr: *mut u8, lay: Layout) -> Result<(), ()> {
		let p = self.base.load(Ordering::Relaxed);
		let off = (addr as isize - p as isize) / Self::CHUNK_SIZE as isize;

		// If pointer is within this sector, mark associated chunk(s) available.
		if (0..64).contains(&off) {
			let num_bits = lay.size().div_ceil(Sector::CHUNK_SIZE);
			let r = !(((!0) >> (PTR_BITS - num_bits)) << off);
			let map = self.slots.load(Ordering::Acquire);

			let res = self.slots.compare_exchange(map, map & r, Ordering::AcqRel, Ordering::Relaxed);
			match res {
				Ok(_) => Ok(()),
				Err(_) => Err(()),
			}
		} else {
			Err(())
		}
	}
}


/* Existing allocator used for static binary crate.
*/

/// Simplest no-logic allocator using mmap exclusively.
///
pub struct SimpleAlloc(AtomicUsize);
impl SimpleAlloc {
	pub const fn new() -> Self {
		SimpleAlloc(AtomicUsize::new(0))
	}
}
unsafe impl GlobalAlloc for SimpleAlloc {
	unsafe fn alloc(&self, lay: Layout) -> *mut u8 {
		let p = unsafe { syscall!( Sysno::mmap,
			0,
			lay.size(),
			PROT_READ | PROT_WRITE,
			MAP_ANONYMOUS | MAP_PRIVATE,
			0,
			0
		)};

		let p = p.map_or(core::ptr::null_mut(), |v| v as *mut u8);

		if !p.is_null() {
			self.0.fetch_add(1, Ordering::Relaxed);
		}

		p as _
	}
	unsafe fn dealloc(&self, ptr: *mut u8, lay: Layout) {
		_ = unsafe { syscall!(Sysno::munmap, ptr as usize, lay.size()) };
		self.0.fetch_sub(0, Ordering::Relaxed);
	}
}
