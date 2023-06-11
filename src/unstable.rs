use core::cmp::Ordering;
use core::mem;
use core::ptr;

/// Sort `v` **without** preserving initial order of equal elements.
///
/// - Guaranteed O(N * log(N)) worst case perf
/// - No adaptiveness
/// - Branch miss-prediction not affected by outcome of comparison function
///
/// If `T: Ord` does not implement a total order the resulting order is
/// unspecified. All original elements will remain in `v` and any possible modifications via
/// interior mutability will be observable. Same is true if `T: Ord` panics.
#[inline(always)]
pub fn sort<T: Ord>(v: &mut [T]) {
    unstable_sort(v, |a, b| a.lt(b))
}

/// Sort `v` **without** preserving initial order of equal elements by comparison function
/// `compare`.
///
/// Same behavior as [`sort`]
#[inline(always)]
pub fn sort_by<T, F: FnMut(&T, &T) -> Ordering>(v: &mut [T], mut compare: F) {
    unstable_sort(v, |a, b| compare(a, b) == Ordering::Less);
}

#[inline(always)]
fn unstable_sort<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], mut is_less: F) {
    if mem::size_of::<T>() == 0 {
        return;
    }

    let len = v.len();

    // Inline the check for len < 2. This happens a lot, instrumenting the Rust compiler suggests
    // len < 2 accounts for 94% of its calls to `slice::sort`.
    if len < 2 {
        return;
    }

    // SAFETY: We just checked that len >= 2.
    unsafe {
        heapsort(v, &mut is_less);
    }
}

/// Sorts `v` using heapsort, which guarantees *O*(*n* \* log(*n*)) worst-case.
///
/// Never inline this, it sits the main hot-loop in `recurse` and is meant as unlikely algorithmic
/// fallback.
///
/// SAFETY: The caller has to guarantee that `v.len()` >= 2.
#[inline(never)]
unsafe fn heapsort<T, F>(v: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if v.len() < 2 {
        // This helps prove things to the compiler. That we checked earlier.
        // SAFETY: This function is only called if len >= 2.
        unsafe {
            core::hint::unreachable_unchecked();
        }
    }

    let len = v.len();

    // Build the heap in linear time.
    for i in (0..len / 2).rev() {
        sift_down(v, i, is_less);
    }

    // Pop maximal elements from the heap.
    for i in (1..len).rev() {
        v.swap(0, i);
        sift_down(&mut v[..i], 0, is_less);
    }
}

// This binary heap respects the invariant `parent >= child`.
//
// SAFETY: The caller has to guarantee that node < `v.len()`.
#[inline(never)]
unsafe fn sift_down<T, F>(v: &mut [T], mut node: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if node >= v.len() {
        // This helps prove things to the compiler. That we checked earlier.
        // SAFETY: This function is only called if node < `v.len()`.
        unsafe {
            core::hint::unreachable_unchecked();
        }
    }

    let len = v.len();

    let arr_ptr = v.as_mut_ptr();

    loop {
        // Children of `node`.
        let mut child = 2 * node + 1;
        if child >= len {
            break;
        }

        // SAFETY: The invariants and checks guarantee that both node and child are in-bounds.
        unsafe {
            // Choose the greater child.
            if child + 1 < len {
                // We need a branch to be sure not to out-of-bounds index,
                // but it's highly predictable.  The comparison, however,
                // is better done branchless, especially for primitives.
                child += is_less(&*arr_ptr.add(child), &*arr_ptr.add(child + 1)) as usize;
            }

            // Stop if the invariant holds at `node`.
            if !is_less(&*arr_ptr.add(node), &*arr_ptr.add(child)) {
                break;
            }

            // Swap `node` with the greater child, move one step down, and continue sifting.
            // Same as v.swap_unchecked(node, child); which is unstable.
            ptr::swap(arr_ptr.add(node), arr_ptr.add(child))
        }

        node = child;
    }
}
