//! The tiny_sort crate provides two sort implementations [`stable::sort`] and [`unstable::sort`].
//! The crate is `no_std` and both versions can be disabled via features, by setting
//! `default-features = false`. [`stable::sort`] requires `alloc`, [`unstable::sort`] doesn't. In
//! addition to the default sort interface that requires that `T` implements [`Ord`],
//! [`stable::sort_by`] [`unstable::sort_by`] can be used to sort with a custom comparison function.
//!
//! Use these sort implementations if you care about binary-size more than you care about
//! performance. Otherwise use `slice::sort` and `slice::sort_unstable`.
//!
//! See the [README](https://github.com/Voultapher/tiny-sort-rs) for information about binary-size
//! and run-time.

#![no_std]

#[cfg(feature = "stable")]
pub mod stable;

#[cfg(feature = "unstable")]
pub mod unstable;
