use std::ops::Range;

use crate::hardware::BufferHandle;
use crate::hardware::Hardware;

#[derive(Debug, Clone)]
pub struct BufferSlice {
    pub handle: BufferHandle,
    pub range: Range<u64>,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub handle: BufferHandle,
    data: Vec<u8>,
}

impl Buffer {
    pub fn new(handle: BufferHandle) -> Self {
        Self {
            handle,
            data: Vec::new(),
        }
    }

    pub fn slice(&self, range: Range<u64>) -> BufferSlice {
        BufferSlice {
            handle: self.handle,
            range,
        }
    }

    pub fn full(&self) -> BufferSlice {
        BufferSlice {
            handle: self.handle,
            range: 0..self.data.capacity() as u64,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
		self.data.extend_from_slice(data);
    }

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn capacity(&self) -> u64 {
        self.data.capacity() as u64
    }

    pub fn flush(&mut self, hardware: &mut impl Hardware) {
        hardware.write_buffer(self.handle, &self.data);
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // Mock implementation of the Hardware trait for testing
    struct MockHardware {
        buffers_written: RefCell<Vec<(BufferHandle, Vec<u8>)>>,
    }

    impl MockHardware {
        fn new() -> Self {
            MockHardware {
                buffers_written: RefCell::new(Vec::new()),
            }
        }
    }

    impl Hardware for MockHardware {
        fn write_buffer(&mut self, handle: BufferHandle, data: &[u8]) {
            self.buffers_written
                .borrow_mut()
                .push((handle, data.to_vec()));
        }
    }

    #[test]
    fn test_buffer_new() {
        let handle = BufferHandle { id: 1 };
        let buffer = Buffer::new(handle);
        assert_eq!(buffer.handle, handle);
        assert_eq!(buffer.data.len(), 0);
        assert_eq!(buffer.offset, 0);
    }

    #[test]
    fn test_buffer_write() {
        let handle = BufferHandle { id: 2 };
        let mut buffer = Buffer::new(handle);
        let data = vec![1, 2, 3, 4];
        buffer.write(&data);
        assert_eq!(buffer.data, data);
        assert_eq!(buffer.offset, 4);
    }

    #[test]
    fn test_buffer_slice() {
        let handle = BufferHandle { id: 3 };
        let buffer = Buffer::new(handle);
        let slice = buffer.slice(1..3);
        assert_eq!(slice.handle, handle);
        assert_eq!(slice.range, 1..3);
    }

    #[test]
    fn test_buffer_full() {
        let handle = BufferHandle { id: 4 };
        let buffer = Buffer::new(handle);
        let slice = buffer.full();
        assert_eq!(slice.handle, handle);
        assert_eq!(slice.range, 0..0);
    }
    #[test]
    fn test_buffer_flush() {
        let handle = BufferHandle { id: 5 };
        let mut buffer = Buffer::new(handle);
        let mut hardware = MockHardware::new();

        buffer.write(&[10, 20, 30]);
        buffer.flush(&mut hardware);

        assert_eq!(buffer.offset, 0);
        assert!(buffer.data.is_empty());

		{
			let written = hardware.buffers_written.borrow();
			assert_eq!(written.len(), 1);
			assert_eq!(written[0].0, handle);
			assert_eq!(written[0].1, vec![10, 20, 30]);
		}

        // Try writing again after flush
        buffer.write(&[40, 50, 60]);
		assert_eq!(buffer.len(), 3);
        buffer.flush(&mut hardware);

        assert_eq!(buffer.offset, 0);
        assert!(buffer.data.is_empty());

        let written = hardware.buffers_written.borrow();
        assert_eq!(written.len(), 2);
        assert_eq!(written[1].0, handle);
        assert_eq!(written[1].1, vec![40, 50, 60]);
    }

    #[test]
    fn test_buffer_begin() {
        let handle = BufferHandle { id: 6 };
        let mut buffer = Buffer::new(handle);
        buffer.write(&[5, 6, 7]);
        buffer.begin();
        assert_eq!(buffer.offset, 0);
        // Data should remain unchanged until flushed
        assert_eq!(buffer.data, vec![5, 6, 7]);
    }

    #[test]
    fn test_buffer_len_capacity() {
        let handle = BufferHandle { id: 7 };
        let buffer = Buffer::new(handle);
        assert_eq!(buffer.len(), 0);
        // Capacity is implementation-defined, so we test that it's at least len
        assert!(buffer.capacity() >= buffer.len());
    }
}
