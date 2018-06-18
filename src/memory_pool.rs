use std::sync::mpsc;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use arc_recycled::{
    ArcRecycled,
    Recycle,
};

/// Boxed closure that creates objects of type T
pub type CreateFn<T> = Box<Fn() -> T>;

/// Memory pool structure
#[allow(missing_debug_implementations)]
pub struct MemoryPool<T> {
    size: AtomicUsize,
    max: usize,
    receiver: mpsc::Receiver<Option<T>>,
    sender: mpsc::Sender<Option<T>>,
    creator: CreateFn<T>,
}

impl<T: Recycle> MemoryPool<T> {
    /// Constructor, must take intial size and maximum size.
    /// The creator closure is used to initialize the mem slots
    /// # Panics
    /// This function will panic if size > max
    pub fn create_with(size: usize, max: usize, creator: CreateFn<T>) -> MemoryPool<T> {
        assert!(size <= max);
        let (tx, rx) = mpsc::channel();
        for _ in 0..size {
            tx.send(Some(creator())).unwrap()
        }

        MemoryPool {
            size: AtomicUsize::new(size),
            max,
            receiver: rx,
            sender: tx,
            creator,
        }
    }

    /// This function returns a memory slot from the memory pool
    /// # Panics
    /// This function will panic if it needs to allocate more than max
    pub fn get(&self) -> ArcRecycled<T> {
        loop {
            /// Try to get a mem_slot
            match self.receiver.try_recv() {
                /// If got one wrap and return it
                Ok(Some(mem_slot)) => {
                    return ArcRecycled::new(mem_slot, self.sender.clone());
                }

                /// If got None, keep trying
                Ok(None) => {
                    self.size.fetch_sub(1, Ordering::Relaxed);
                }

                /// If channel is empty try to create a new memory slot
                /// If we have place this works, if not, it panics!
                Err(mpsc::TryRecvError::Empty) => {
                    if self.size.fetch_add(1, Ordering::Relaxed) < self.max {
                        return ArcRecycled::new((self.creator)(), self.sender.clone());
                    }
                    else {
                        panic!("Exceeded memory pool limit");
                    }
                }

                /// Unreachable
                Err(_) => {
                    unreachable!("If the memory pool is alive, the channel cannot be disconnected")
                }
            }
        }
    }

    /// This function returns a memory slot from the memory pool
    /// if size does not exceed max. returns None otherwise
    pub fn try_get(&self) -> Option<ArcRecycled<T>> {
        loop {
            /// Try to get a mem_slot
            match self.receiver.try_recv() {
                /// If got one wrap and return it
                Ok(Some(mem_slot)) => {
                    return Some(ArcRecycled::new(mem_slot, self.sender.clone()));
                }

                /// If got None, keep trying
                Ok(None) => {
                    self.size.fetch_sub(1, Ordering::Relaxed);
                }

                /// If channel is empty try to create a new memory slot
                /// If we have place this works, if not, it panics!
                Err(mpsc::TryRecvError::Empty) => {
                    if self.size.fetch_add(1, Ordering::Relaxed) < self.max {
                        return Some(ArcRecycled::new((self.creator)(), self.sender.clone()));
                    }
                    else {
                        return None;
                    }
                }

                /// Unreachable
                Err(_) => {
                    unreachable!("If the memory pool is alive, the channel cannot be disconnected")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_test() {
        let mem = MemoryPool::create_with(5, 10, Box::new(|| { Vec::<f64>::with_capacity(20) }));
        let _v1 = mem.get();
        let _v2 = mem.try_get().unwrap();
    }

    #[test]
    fn extra_elements_test() {
        let mem = MemoryPool::create_with(5, 10, Box::new(|| { Vec::<f64>::with_capacity(20) }));
        let mut vecs = vec![];
        for _ in 0..10 {
            vecs.push(mem.get());
        }
    }

    #[test]
    fn recycling_test() {
        let mem = MemoryPool::create_with(5, 10, Box::new(|| { Vec::<f64>::with_capacity(20) }));
        /// First use of all 10 elements
        {
            let mut vecs = vec![];
            for _ in 0..10 {
                vecs.push(mem.get());
            }
        }

        /// Second use of all 10 elements
        {
            let mut vecs = vec![];
            for _ in 0..10 {
                vecs.push(mem.get());
            }
        }
    }

    #[test]
    #[should_panic]
    fn too_many_elements_test() {
        let mem = MemoryPool::create_with(5, 10, Box::new(|| { Vec::<f64>::with_capacity(20) }));
        let mut vecs = vec![];
        for _ in 0..11 {
            vecs.push(mem.get());
        }
    }

    #[test]
    fn too_many_elements_try_test() {
        let mem = MemoryPool::create_with(5, 10, Box::new(|| { Vec::<f64>::with_capacity(20) }));
        let mut vecs = vec![];
        for _ in 0..10 {
            vecs.push(mem.get());
        }
        assert!(mem.try_get().is_none());
    }
}
