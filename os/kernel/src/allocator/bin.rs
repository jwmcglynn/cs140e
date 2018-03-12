use std::fmt;
use std::mem;
use alloc::heap::{AllocErr, Layout};
use console::kprintln;

use allocator::util::*;
use allocator::linked_list::LinkedList;
use allocator::AllocStats;

const FIRST_BIN_SIZE: usize = 1 << 3;
const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

fn previous_power_of_two(num: usize) -> usize {
    if num < 2 {
        0
    } else {
        1 << (USIZE_BITS - num.leading_zeros() as usize - 1)
    }
}

fn bin_size(size: usize) -> usize {
    if size < FIRST_BIN_SIZE {
        FIRST_BIN_SIZE
    } else {
        size.next_power_of_two()
    }
}

fn size_to_bin_number(size: usize) -> usize {
    if size < FIRST_BIN_SIZE {
        0
    } else {
        (bin_size(size).trailing_zeros() - FIRST_BIN_SIZE.trailing_zeros()) as usize
    }
}

fn has_alignment(size: *mut usize, align: usize) -> bool {
    (size as usize) % align == 0
}

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    bins: [LinkedList; 64],

    max_bin_size: usize,
    unallocated_current: usize,
    unallocated_end: usize,

    // Stats.
    alloc_count: usize,
    mem_use_actual: usize,
    mem_use_effective: usize,
    internal_fragmentation_bytes: usize,
    external_fragmentation_bytes: usize,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let bins: [LinkedList; 64] = [LinkedList::new(); 64];

        let max_bin_size = previous_power_of_two(end - start);
        Allocator { bins, max_bin_size, unallocated_current: start,
                    unallocated_end: end, alloc_count: 0, mem_use_actual: 0,
                    mem_use_effective: 0, internal_fragmentation_bytes: 0,
                    external_fragmentation_bytes: 0}
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if layout.size() > self.max_bin_size {
            return Err(AllocErr::Unsupported{ details: "Allocation too large." });
        }

        // Try to service the request from the bin's free list.
        let ref mut bin = self.bins[size_to_bin_number(layout.size())];
        for mut node in bin.iter_mut() {
            if has_alignment(node.value(), layout.align()) {
                // Calculate stats.
                self.alloc_count += 1;
                self.mem_use_actual += layout.size();
                self.mem_use_effective += bin_size(layout.size());
                // Don't count internal fragmentation again, it's already there.
                self.external_fragmentation_bytes += bin_size(layout.size()) - layout.size();

                return Ok(node.pop() as *mut u8);
            }
        }

        let bin_size: usize = bin_size(layout.size());

        let start = align_up(self.unallocated_current, layout.align());
        if self.unallocated_end.saturating_sub(bin_size) < start {
            Err(AllocErr::Exhausted{ request: layout })
        } else {
            let align_padding = start - self.unallocated_current;
            let alloc_size = align_padding + bin_size;
            self.unallocated_current = start + bin_size;

            // Calculate stats.
            self.alloc_count += 1;
            self.mem_use_actual += layout.size();
            self.mem_use_effective += alloc_size;
            self.internal_fragmentation_bytes += align_padding;
            self.external_fragmentation_bytes += bin_size - layout.size();

            Ok(start as *mut u8)
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.bins[size_to_bin_number(layout.size())].push(ptr as *mut usize);
        }

        // Calculate stats.
        let bin_size = bin_size(layout.size());
        self.alloc_count -= 1;
        self.mem_use_actual -= layout.size();
        self.mem_use_effective -= bin_size;
        // Internal fragmentation can't be recovered, it persists in the block.
        self.external_fragmentation_bytes -= bin_size - layout.size();
    }
}

impl AllocStats for Allocator {
    fn print_stats(&self) {
        kprintln!("Memory Use: {} bytes", self.mem_use_actual);
        kprintln!("Memory Use (Effective): {} bytes", self.mem_use_effective);
        kprintln!("Outstanding Allocations: {}", self.alloc_count);
        kprintln!("Internal Fragmentation: {} bytes", self.internal_fragmentation_bytes);
        kprintln!("External Fragmentation: {} bytes", self.external_fragmentation_bytes);

        kprintln!("Bins:");
        let mut bin_size = FIRST_BIN_SIZE;
        for i in 0..size_to_bin_number(self.max_bin_size) {
            kprintln!("  bin#{} size={} entries={}", i, bin_size, self.bins[i].iter().count());
            bin_size = bin_size * 2;
        }
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Allocator {{")?;
        writeln!(f, "  max_bin_size: {}", self.max_bin_size)?;
        writeln!(f, "  unallocated_current: {}", self.unallocated_current)?;
        writeln!(f, "  unallocated_end: {}", self.unallocated_end)?;

        let mut bin_size = FIRST_BIN_SIZE;
        for i in 0..size_to_bin_number(self.max_bin_size) {
            writeln!(f, "  bin#{} size={} = {:#?}", i, bin_size, self.bins[i])?;
            bin_size = bin_size * 2;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_power_of_two() {
        assert_eq!(previous_power_of_two(0), 0);
        assert_eq!(previous_power_of_two(1), 0);
        assert_eq!(previous_power_of_two(5), 4);
        assert_eq!(previous_power_of_two(4), 4);
        assert_eq!(previous_power_of_two(96), 64);
        assert_eq!(previous_power_of_two(1 << 32), 1 << 32);
        assert_eq!(previous_power_of_two(1 << 14), 1 << 14);
        assert_eq!(previous_power_of_two((1 << 30 - 1)), 1 << 29);
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
