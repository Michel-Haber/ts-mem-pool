use std::sync::Arc;
use std::sync::mpsc;
use std::ops::Deref;
use std::ops::DerefMut;

/// This trait is required to use
/// a type in the memory pool
pub trait Recycle {
    /// Render data of the object usable
    ///
    /// # Example
    /// ```
    /// extern crate a_memory_pool;
    /// use a_memory_pool::*;
    ///
    /// struct MyType {
    ///     /* Some fields */
    /// }
    ///
    /// impl MyType {
    ///     fn reset(&mut self) {
    ///         /* Do Something */
    ///     }
    /// }
    ///
    /// impl Recycle for MyType {
    ///     fn recycle(&mut self) {
    ///         self.reset();
    ///     }
    /// }
    /// ```
    fn recycle(&mut self);
}

/// A smart pointer that returns memory to
/// its owner when it is dropped
#[derive(Debug)]
pub struct ArcRecycled<T: Recycle> {
    /// Data content
    content: Option<Arc<T>>,

    /// Owner's reception channel
    owner: mpsc::Sender<Option<T>>,
}

impl<T: Recycle> ArcRecycled<T> {
    /// Constructor function, takes the memory slot and the channel to the pool
    pub fn new(data: T, owner_channel: mpsc::Sender<Option<T>>) -> ArcRecycled<T> {
        ArcRecycled {
            content: Some(Arc::new(data)),
            owner: owner_channel,
        }
    }

    /// Returns a reference to the content.
    /// Deref can be used instead
    pub fn get_ref(&self) -> &T {
        &self.content.as_ref().expect("Missing content")
    }

    /// Returns a mutable reference to the
    /// content, if this is the only owner,
    /// if not returns None
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let arc = self.content.as_mut().expect("Missing content");
        Arc::get_mut(arc)
    }
}

impl<T: Recycle> Clone for ArcRecycled<T> {
    /// Normal clone, but it is better
    /// explicited here.
    fn clone(&self) -> ArcRecycled<T> {
        ArcRecycled {
            content: self.content.clone(),
            owner: self.owner.clone(),
        }
    }
}

impl<T: Recycle> Drop for ArcRecycled<T> {
    /// If strong_count is at 1, return data to owner
    fn drop(&mut self) {
        match self.content.take() {
            /// Value still there, memory should be returned to owner
            Some(value) => {
                match Arc::try_unwrap(value) {
                    /// If unwrapped, send the value back to the pool
                    /// If this fails, we have lost our pool. The mem
                    /// slot will be simply dropped
                    Ok(mut mem_slot) => {
                        mem_slot.recycle();
                        let _ = self.owner.send(Some(mem_slot));
                    }
                    /// If not unwrapped, you are not truly the last owner
                    /// Refill the content
                    Err(arc)=> {
                        self.content = Some(arc);
                    }
                }
            }
            /// Value taken, this is just an empty shell
            /// Tell the owner it is taken from him
            /// If this fails, we have lost our pool. The mem
            /// slot will be simply dropped
            None => {
                let _ = self.owner.send(None);
            }
        }
    }
}

impl<T: Recycle> Deref for ArcRecycled<T> {
    type Target = T;
    /// Give a reference directly to the
    /// innermost content
    fn deref(&self) -> &T {
        &self.content.as_ref().expect("Missing content")
    }
}

impl<T: Recycle> DerefMut for ArcRecycled<T> {
    /// Give a mutable reference directly to the
    /// innermost content
    ///
    /// # Panics
    ///
    /// Dereferencing a shared pointer
    /// mutably will cause a panic.
    /// This trait should only be used for
    /// initialization. Any further change should
    /// be done with get_mut()
    fn deref_mut(&mut self) -> &mut T {
        let arc = self.content.as_mut().expect("Missing content");
        Arc::get_mut(arc).expect("Mutable Deref not allowed")
    }
}

impl<T> Recycle for Vec<T> {
    fn recycle(&mut self) {
        self.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref_usage_test() {
        let (tx, _) = mpsc::channel();
        let mut rec = ArcRecycled::new(Vec::<f64>::with_capacity(50), tx);

        for _ in 0..10 {
            rec.push(5.0);
        }
    }

    #[test]
    #[should_panic]
    fn multiple_ref_write_test() {
        let (tx, _) = mpsc::channel();
        let mut rec = ArcRecycled::new(Vec::<f64>::with_capacity(50), tx);
        let _rec2 = rec.clone();

        for _ in 0..10 {
            rec.push(5.0);
        }
    }

    #[test]
    fn data_return_test() {
        /// Channel to recv recycled data
        let (tx, rx) = mpsc::channel();

        /// New scope for adding values
        {
            let mut rec = ArcRecycled::new(Vec::<f64>::with_capacity(50), tx);
            for _ in 0..10 {
                rec.push(5.0);
            }
        }

        /// the recycled vec
        let new_val = rx.recv().unwrap().unwrap();

        assert_eq!(new_val.len(), 0);
        assert_eq!(new_val.capacity(), 50);
    }

    #[test]
    fn no_data_return_test() {
        /// Channel to recv recycled data
        let (tx, rx) = mpsc::channel();
        let mut rec = ArcRecycled::new(Vec::<f64>::with_capacity(50), tx);
        for _ in 0..10 {
            rec.push(5.0);
        }

        /// New scope for cloning
        {
            let _rec2 = rec.clone();
        }

        /// No recycled data
        let _new_val = rx.try_recv().unwrap_err();

        assert_eq!(rec.len(), 10);
        assert_eq!(rec.capacity(), 50);
    }
}
