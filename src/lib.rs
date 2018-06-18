//! This simple crate provides a thread-safe memory pool
//! It is based on smart pointers, that, when dropped,
//! return ownership of their memory slot to the pool.

#![allow(unknown_lints)]
#![allow(renamed_and_removed_lints)]
#![allow(unused_doc_comment)]
#![allow(unused_doc_comments)]

#![warn(missing_copy_implementations,
        missing_debug_implementations,
        missing_docs,
        trivial_numeric_casts,
        unsafe_code,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications,
        unreachable_pub)]

/// Definition of the smart pointer
pub mod arc_recycled;

/// Definition of the pool structure
pub mod memory_pool;

pub use memory_pool::MemoryPool;
pub use memory_pool::CreateFn;

pub use arc_recycled::ArcRecycled;
pub use arc_recycled::Recycle;
