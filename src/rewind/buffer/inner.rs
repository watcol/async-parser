use std::collections::{vec_deque, BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Buffer {
    offset: usize,
    checkpoints: BTreeMap<usize, u8>,
    record: VecDeque<u8>,
}

impl Buffer {
    pub fn push(&mut self, buf: &[u8]) {
        self.record.extend(buf);
    }

    pub fn is_recording(&self) -> bool {
        !self.checkpoints.is_empty()
    }

    pub fn first_checkpoint(&self) -> usize {
        self.checkpoints
            .keys()
            .next()
            .copied()
            .unwrap_or_else(|| self.end())
    }

    pub fn pop(&mut self, pos: usize, bytes: usize) -> (usize, vec_deque::Drain<'_, u8>) {
        let end = self.end();
        assert!(pos == self.offset);
        assert!(end > pos);
        let bytes = match self.checkpoints.keys().next() {
            Some(&ck) if ck < self.offset + bytes => ck - self.offset,
            Some(_) => bytes,
            None if end < self.offset + bytes => end - self.offset,
            None => bytes,
        };
        self.offset += bytes;
        (bytes, self.record.drain(..bytes))
    }

    pub fn pop_ref(&mut self, pos: usize, bytes: usize) -> (usize, vec_deque::Iter<'_, u8>) {
        let end = self.end();
        assert!(pos >= self.offset);
        assert!(end > pos);
        let bytes = if end < pos + bytes { end - pos } else { bytes };
        let offset = pos - self.offset;
        (bytes, self.record.range(offset..(offset + bytes)))
    }

    pub fn end(&self) -> usize {
        self.offset + self.record.len()
    }

    pub fn set_checkpoint(&mut self, offset: usize) {
        if self.checkpoints.is_empty() {
            self.offset = offset;
        }
        match self.checkpoints.get_mut(&offset) {
            Some(s) => {
                *s += 1;
            }
            None => {
                self.checkpoints.insert(offset, 1);
            }
        }
    }

    pub fn rewind(&mut self, cp: usize) {
        match self.checkpoints.get_mut(&cp) {
            Some(s) if *s == 1 => {
                self.checkpoints.remove(&cp);
            }
            Some(s) => {
                *s -= 1;
            }
            None => {
                panic!("No such checkpoint.");
            }
        }
    }
}
