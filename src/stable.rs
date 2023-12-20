use core::cmp::Ordering;
use core::mem::{self, MaybeUninit};
use core::ptr;

extern crate alloc;

use alloc::alloc::{alloc, dealloc, Layout};

/// Sort `v` preserving initial order of equal elements.
///
/// - Guaranteed O(N * log(N)) worst case perf
/// - No adaptiveness
/// - Branch miss-prediction not affected by outcome of comparison function
/// - Uses `v.len()` auxiliary memory.
///
/// If `T: Ord` does not implement a total order the resulting order is
/// unspecified. All original elements will remain in `v` and any possible modifications via
/// interior mutability will be observable. Same is true if `T: Ord` panics.
///
/// Panics if allocating the auxiliary memory fails.
#[inline(always)]
pub fn sort<T: Ord>(v: &mut [T]) {
    stable_sort(v, |a, b| a.lt(b))
}

/// Sort `v` preserving initial order of equal elements by comparison function `compare`.
///
/// Same behavior as [`sort`].
#[inline(always)]
pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(v: &mut [T], mut compare: F) {
    stable_sort(v, |a, b| compare(a, b) == Ordering::Less);
}

/// Sort `v` preserving initial order of equal elements by key extraction function `f`.
///
/// Same behavior as [`sort`]
#[inline(always)]
pub fn sort_by_key<T, K, F>(v: &mut [T], mut f: F)
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    stable_sort(v, |a, b| f(a).lt(&f(b)));
}

#[inline(always)]
fn stable_sort<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], mut is_less: F) {
    if mem::size_of::<T>() == 0 {
        return;
    }

    let len = v.len();

    // Inline the check for len < 2. This happens a lot, instrumenting the Rust compiler suggests
    // len < 2 accounts for 94% of its calls to `slice::sort`.
    if len < 2 {
        return;
    }

    // SAFETY: We checked that len is > 0 and that T is not a ZST.
    unsafe {
        mergesort_main(v, &mut is_less);
    }
}

/// The core logic should not be inlined.
///
/// SAFETY: The caller has to ensure that len is > 0 and that T is not a ZST.
#[inline(never)]
unsafe fn mergesort_main<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], is_less: &mut F) {
    // While it would be nice to have a merge implementation that only requires N / 2 auxiliary
    // memory. Doing so would make the merge implementation significantly more complex and

    // SAFETY: See function safety description.
    let buf = unsafe { BufGuard::new(v.len()) };

    // SAFETY: `scratch` has space for `v.len()` writes. And does not alias `v`.
    unsafe {
        mergesort_core(v, buf.buf_ptr.as_ptr(), is_less);
    }
}

/// Tiny recursive top-down merge sort optimized for binary size. It has no adaptiveness whatsoever,
/// no run detection, etc.
///
/// Buffer as pointed to by `scratch` must have space for `v.len()` writes. And must not alias `v`.
#[inline(always)]
unsafe fn mergesort_core<T, F: FnMut(&T, &T) -> bool>(
    v: &mut [T],
    scratch_ptr: *mut T,
    is_less: &mut F,
) {
    let len = v.len();

    if len > 2 {
        // SAFETY: `mid` is guaranteed in-bounds. And caller has to ensure that `scratch_ptr` can
        // hold `v.len()` values.
        unsafe {
            let mid = len / 2;
            // Sort the left half recursively.
            mergesort_core(v.get_unchecked_mut(..mid), scratch_ptr, is_less);
            // Sort the right half recursively.
            mergesort_core(v.get_unchecked_mut(mid..), scratch_ptr, is_less);
            // Combine the two halves.
            merge(v, scratch_ptr, is_less, mid);
        }
    } else if len == 2 {
        // Branchless swap the two elements. This reduces the recursion depth and improves
        // perf significantly at a small binary-size cost. Trades ~10% perf boost for integers
        // for ~50 bytes in the binary.
        let should_swap = is_less(&v[1], &v[0]);
        // SAFETY: We checked the len, the pointers we create are valid and don't overlap.
        unsafe {
            branchless_swap(&mut v[1], &mut v[0], should_swap);
        }
    }
}

/// Branchless merge function.
///
/// SAFETY: The caller must ensure that `scratch_ptr` is valid for `v.len()` writes. And that mid is
/// in-bounds.
#[inline(always)]
unsafe fn merge<T, F>(v: &mut [T], scratch_ptr: *mut T, is_less: &mut F, mid: usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = v.len();
    debug_assert!(mid > 0 && mid < len);

    let len = v.len();

    // Indexes to track the positions while merging.
    let mut l = 0;
    let mut r = mid;

    // SAFETY: No matter what the result of is_less is we check that l and r remain in-bounds and if
    // is_less panics the original elements remain in `v`.
    unsafe {
        let arr_ptr = v.as_ptr();

        for i in 0..len {
            let left_ptr = arr_ptr.add(l);
            let right_ptr = arr_ptr.add(r);

            let is_lt = !is_less(&*right_ptr, &*left_ptr);
            let copy_ptr = if is_lt { left_ptr } else { right_ptr };
            ptr::copy_nonoverlapping(copy_ptr, scratch_ptr.add(i), 1);

            l += is_lt as usize;
            r += !is_lt as usize;

            // As long as neither side is exhausted merge left and right elements.
            if ((l == mid) as u8 + (r == len) as u8) != 0 {
                break;
            }
        }

        // The left or right side is exhausted, drain the right side in one go.
        let copy_ptr = if l == mid {
            arr_ptr.add(r)
        } else {
            arr_ptr.add(l)
        };
        let i = l + (r - mid);
        ptr::copy_nonoverlapping(copy_ptr, scratch_ptr.add(i), len - i);

        // Now that scratch_ptr holds the full merged content, write it back on-top of v.
        ptr::copy_nonoverlapping(scratch_ptr, v.as_mut_ptr(), len);
    }
}

/// Swap two values in array pointed to by a_ptr and b_ptr if b is less than a.
#[inline(always)]
pub unsafe fn branchless_swap<T>(a_ptr: *mut T, b_ptr: *mut T, should_swap: bool) {
    // SAFETY: the caller must guarantee that `a_ptr` and `b_ptr` are valid for writes
    // and properly aligned, and part of the same allocation, and do not alias.

    // This is a branchless version of swap if.
    // The equivalent code with a branch would be:
    //
    // if should_swap {
    //     ptr::swap_nonoverlapping(a_ptr, b_ptr, 1);
    // }

    // Give ourselves some scratch space to work with.
    // We do not have to worry about drops: `MaybeUninit` does nothing when dropped.
    let mut tmp = MaybeUninit::<T>::uninit();

    // The goal is to generate cmov instructions here.
    let a_swap_ptr = if should_swap { b_ptr } else { a_ptr };
    let b_swap_ptr = if should_swap { a_ptr } else { b_ptr };

    ptr::copy_nonoverlapping(b_swap_ptr, tmp.as_mut_ptr(), 1);
    ptr::copy(a_swap_ptr, a_ptr, 1);
    ptr::copy_nonoverlapping(tmp.as_ptr(), b_ptr, 1);
}

// SAFETY: The caller has to ensure that Option is Some, UB otherwise.
unsafe fn unwrap_unchecked<T>(opt_val: Option<T>) -> T {
    match opt_val {
        Some(val) => val,
        None => {
            // SAFETY: See function safety description.
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    }
}

// Extremely basic versions of Vec.
// Their use is super limited and by having the code here, it allows reuse between the sort
// implementations.
struct BufGuard<T> {
    buf_ptr: ptr::NonNull<T>,
    capacity: usize,
}

impl<T> BufGuard<T> {
    // SAFETY: The caller has to ensure that len is not 0 and that T is not a ZST.
    unsafe fn new(len: usize) -> Self {
        debug_assert!(len > 0 && mem::size_of::<T>() > 0);

        // SAFETY: See function safety description.
        let layout = unsafe { unwrap_unchecked(Layout::array::<T>(len).ok()) };

        // SAFETY: We checked that T is not a ZST.
        let buf_ptr = unsafe { alloc(layout) as *mut T };

        if buf_ptr.is_null() {
            panic!("allocation failure");
        }

        Self {
            buf_ptr: ptr::NonNull::new(buf_ptr).unwrap(),
            capacity: len,
        }
    }
}

impl<T> Drop for BufGuard<T> {
    fn drop(&mut self) {
        // SAFETY: We checked that T is not a ZST.
        unsafe {
            dealloc(
                self.buf_ptr.as_ptr() as *mut u8,
                Layout::array::<T>(self.capacity).unwrap(),
            );
        }
    }
}
