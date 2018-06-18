# Memory pool
This is a simple crate meant to allow sharing data allocated in a memory pool.  
Before sharing the data, it can be written, but once shared it becomes read-only.  
Dropped data is returned to the memory pool where it is recycled and can be used
to share other information

# Performance
This approach only gains performance, when working without memory constraints,
when the size of a memory slot reaches sizes near 50 000 bytes.
Near 4 000 000 bytes, the time gain becomes sizeable, and in this range, this
crate would be truly useful.

# TODO
- Add detach method for ArcRecycled
- Add Size methods for MemoryPool
- Add attach method for MemoryPool
