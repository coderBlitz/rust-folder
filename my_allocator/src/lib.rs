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

const ALLOX_POOL_SIZE: usize = 4;
const PTR_BITS: usize = core::mem::size_of::<usize>() * 8;

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
					println!("Found available sector at [{_i}]");
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
}
unsafe impl GlobalAlloc for Allox {
	unsafe fn alloc(&self, lay: Layout) -> *mut u8 {
		// Iterate existing sectors
		for i in 0..self.num_sectors.load(Ordering::Relaxed) {
			let sec = &self.sectors[i];
			// Load pointer and check if null before trying logic
			let p = sec.load(Ordering::Relaxed);
			if !p.is_null() {
				// Check if not at max, then increment.
				let v = (*p).viewers.load(Ordering::Acquire);
				if v < usize::MAX {
					let r = (*p).viewers.compare_exchange(v, v+1, Ordering::AcqRel, Ordering::Relaxed);

					// If viewers successfully incremented, try to find space in sector.
					if let Ok(_) = r {
						println!("Searching in sector [{i}]..");
						// Try to get memory for layout.
						let res = (*p).request_mem(lay);

						// Decrement viewer since memory request is complete
						(*p).viewers.fetch_sub(1, Ordering::Release);

						// If request was successful, return ptr.
						if let Ok(ptr) = res {
							self.total_allocs.fetch_add(1, Ordering::Relaxed);
							return ptr
						}
					}
				}
			}
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
		// Iterate existing sectors
		for _s in self.sectors.iter() {
		}

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
