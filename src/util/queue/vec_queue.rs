use std::marker::PhantomData;

pub struct LocalQueue<'a, T> {
    queue: Vec<T>,
    id: usize,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> LocalQueue<'a, T>{
    pub fn new(id: usize) -> Self {
        LocalQueue {
            queue: vec![],
            id,
            phantom: PhantomData
        }
    }

    pub fn enqueue(&mut self, v: T) {
        self.queue.push(v);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.queue.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn reset(&mut self) {
        self.queue.clear()
    }
}