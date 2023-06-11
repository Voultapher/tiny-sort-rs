#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "stable")]
pub mod stable;

#[cfg(feature = "unstable")]
pub mod unstable;
