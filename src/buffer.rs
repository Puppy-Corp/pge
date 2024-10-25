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
    offset: u64,
}

impl Buffer {
    pub fn new(handle: BufferHandle) -> Self {
        Self {
            handle,
            data: Vec::new(),
            offset: 0,
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
            range: 0..self.data.len() as u64,
        }
    }
    
    pub fn begin(&mut self) {
        self.offset = 0;
    }

    pub fn write(&mut self, data: &[u8]) {
        let end = self.offset + data.len() as u64;
        if end > self.data.len() as u64 {
            self.data.resize(end as usize, 0);
        }
        self.data[self.offset as usize..end as usize].copy_from_slice(data);
        self.offset = end;
    }

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn capacity(&self) -> u64 {
        self.data.capacity() as u64
    }

	pub fn flush(&mut self, hardware: &mut impl Hardware) {
		hardware.write_buffer(self.handle, &self.data);
		self.offset = 0;
	}
}
