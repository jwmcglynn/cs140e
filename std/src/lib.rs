#![stable(feature = "rust1", since = "1.0.0")]

// Don't link to std. We are std.
#![no_std]

// std may use features in a platform-specific way
#![allow(unused_features)]

// std is implemented with unstable features, many of which are internal
// compiler details that will never be stable
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(alloc_system)]
#![feature(allocator_internals)]
#![feature(allow_internal_unsafe)]
#![feature(allow_internal_unstable)]
#![feature(align_offset)]
#![feature(array_error_internals)]
#![feature(ascii_ctype)]
#![feature(asm)]
#![feature(attr_literals)]
#![feature(box_syntax)]
#![feature(cfg_target_has_atomic)]
#![feature(cfg_target_thread_local)]
#![feature(cfg_target_vendor)]
#![feature(char_error_internals)]
#![feature(char_internals)]
#![feature(collections_range)]
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(core_float)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(exact_size_is_empty)]
#![feature(fs_read_write)]
#![feature(fixed_size_array)]
#![feature(float_from_str_radix)]
#![feature(fn_traits)]
#![feature(fnbox)]
#![feature(fused)]
#![feature(generic_param_attrs)]
#![feature(hashmap_hasher)]
#![feature(heap_api)]
#![feature(i128)]
#![feature(i128_type)]
#![feature(inclusive_range)]
#![feature(int_error_internals)]
#![feature(integer_atomics)]
#![feature(into_cow)]
#![feature(lang_items)]
#![feature(libc)]
#![feature(link_args)]
#![feature(linkage)]
#![feature(macro_reexport)]
#![feature(macro_vis_matcher)]
#![feature(needs_panic_runtime)]
#![feature(never_type)]
#![feature(num_bits_bytes)]
#![feature(old_wrapping)]
#![feature(on_unimplemented)]
#![feature(oom)]
#![feature(optin_builtin_traits)]
#![feature(panic_unwind)]
#![feature(peek)]
#![feature(placement_in_syntax)]
#![feature(placement_new_protocol)]
#![feature(prelude_import)]
#![feature(ptr_internals)]
#![feature(rand)]
#![feature(raw)]
#![feature(repr_align)]
#![feature(rustc_attrs)]
#![feature(sip_hash_13)]
#![feature(slice_bytes)]
#![feature(slice_concat_ext)]
#![feature(slice_internals)]
#![feature(slice_patterns)]
#![feature(staged_api)]
#![feature(stmt_expr_attributes)]
#![feature(str_char)]
#![feature(str_internals)]
#![feature(str_utf16)]
#![feature(termination_trait)]
#![feature(test, rustc_private)]
#![feature(thread_local)]
#![feature(toowned_clone_into)]
#![feature(try_from)]
#![feature(unboxed_closures)]
#![feature(unicode)]
#![feature(untagged_unions)]
#![feature(unwind_attributes)]
#![feature(vec_push_all)]
#![feature(doc_cfg)]
#![feature(doc_masked)]
#![feature(doc_spotlight)]

// TODO: These are additions.
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(pattern)]
#![feature(slice_get_slice)]
#![feature(slice_rsplit)]
#![feature(from_ref)]
#![feature(swap_with_slice)]

// Explicitly import the prelude. The compiler uses this same unstable attribute
// to import the prelude implicitly when building crates that depend on std.
#[prelude_import]
#[allow(unused)]
use prelude::v1::*;

// // Access to Bencher, etc.
// #[cfg(test)] extern crate test;
// #[cfg(test)] extern crate rand;

// We want to re-export a few macros from core but libcore has already been
// imported by the compiler (via our #[no_std] attribute) In this case we just
// add a new crate name so we can attach the re-exports to it.
#[macro_reexport(panic, assert, assert_eq, assert_ne, debug_assert, debug_assert_eq,
                 debug_assert_ne, unreachable, unimplemented, write, writeln, try)]
extern crate core as __core;

// #[macro_use]
// #[macro_reexport(vec, format)]
// extern crate alloc;
// extern crate alloc_system;
extern crate std_unicode;
// #[doc(masked)]
// extern crate libc;

// // We always need an unwinder currently for backtraces
// #[doc(masked)]
// #[allow(unused_extern_crates)]
// extern crate unwind;

// compiler-rt intrinsics
#[doc(masked)]
extern crate compiler_builtins;

// // During testing, this crate is not actually the "real" std library, but rather
// // it links to the real std library, which was compiled from this same source
// // code. So any lang items std defines are conditionally excluded (or else they
// // wolud generate duplicate lang item errors), and any globals it defines are
// // _not_ the globals used by "real" std. So this import, defined only during
// // testing gives test-std access to real-std lang items and globals. See #2912
// #[cfg(test)] extern crate std as realstd;

// The standard macros that are not built-in to the compiler.
#[macro_use]
mod macros;

// The Rust prelude
pub mod prelude;

// Public module declarations and re-exports
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::any;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::cell;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::clone;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::cmp;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::convert;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::default;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::hash;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::intrinsics;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::iter;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::marker;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::mem;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::ops;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::ptr;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::raw;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::result;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::option;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::isize;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i8;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i16;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i32;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::i64;
#[unstable(feature = "i128", issue = "35118")]
pub use core::i128;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::usize;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u8;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u16;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u32;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::u64;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::boxed;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::rc;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::borrow;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::fmt;
#[stable(feature = "rust1", since = "1.0.0")]
pub use core::fmt;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::slice;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::str;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::string;
// #[stable(feature = "rust1", since = "1.0.0")]
// pub use alloc::vec;
#[stable(feature = "rust1", since = "1.0.0")]
pub use std_unicode::char;
#[unstable(feature = "i128", issue = "35118")]
pub use core::u128;

// TODO: This is an addition. This should should actually come from `alloc`.
pub mod str;
pub mod slice;

// pub mod f32;
// pub mod f64;

// #[macro_use]
// pub mod thread;
// pub mod ascii;
// pub mod collections;
// pub mod env;
// pub mod error;
// pub mod ffi;
// pub mod fs;
pub mod io;
// pub mod net;
// pub mod num;
// pub mod os;
// pub mod panic;
// pub mod path;
// pub mod process;
pub mod sync;
// pub mod time;
// pub mod heap;

// // Platform-abstraction modules
// #[macro_use]
// mod sys_common;
// mod sys;

// // Private support modules
// mod panicking;
// mod memchr;

// // The runtime entry point and a few unstable public functions used by the
// // compiler
// pub mod rt;
// // The trait to support returning arbitrary types in the main function
// mod termination;

// #[unstable(feature = "termination_trait", issue = "43301")]
// pub use self::termination::Termination;
