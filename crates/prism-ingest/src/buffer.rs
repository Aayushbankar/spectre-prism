use bytes::BytesMut;
use slab::Slab;

pub struct BufferPool {
    pool: Slab<BytesMut>,
    chunk_size: usize,
}

impl BufferPool {
    pub fn new(capacity: usize, chunk_size: usize) -> Self {
        let mut pool = Slab::with_capacity(capacity);
        for _ in 0..capacity {
            pool.insert(BytesMut::with_capacity(chunk_size));
        }
        Self { pool, chunk_size }
    }

    pub fn get(&mut self) -> Option<(usize, BytesMut)> {
        let key = self.pool.iter().next()?.0;
        let mut buf = self.pool.remove(key);
        buf.reserve(self.chunk_size);
        Some((key, buf))
    }

    pub fn put(&mut self, _key: usize, mut buf: BytesMut) {
        buf.clear();
        self.pool.insert(buf);
    }
}
