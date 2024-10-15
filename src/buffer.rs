#[derive(Debug, Clone, Default)]
pub struct DirtyBuffer {
    pub name: String,
    data: Vec<u8>,
    pub dirty: bool,
    offset: usize,
}

impl DirtyBuffer {
    /// Creates a new DirtyBuffer with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            dirty: false,
            offset: 0,
        }
    }

    /// Returns the current length of valid data in the buffer.
    pub fn len(&self) -> usize {
        self.offset
    }

    /// Extends the buffer with data from the given slice.
    /// Marks the buffer as dirty if any changes are made.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        if self.offset + slice.len() > self.data.len() {
            log::info!(
                "[{}] data is bigger offset: {} slice.len: {} data.len: {}",
                self.name,
                self.offset,
                slice.len(),
                self.data.len()
            );
            self.data.resize(self.offset + slice.len(), 0);
            self.dirty = true;
        }

        let current_slice = &self.data[self.offset..self.offset + slice.len()];
        if current_slice != slice {
            self.data[self.offset..self.offset + slice.len()].copy_from_slice(slice);
            self.dirty = true;
        }
        self.offset += slice.len();
    }

    /// Resets the offset to zero without modifying the underlying data.
    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }

    /// Clears the buffer, removing all data and marking it as dirty.
    pub fn clear(&mut self) {
        self.data.clear();
        self.dirty = true;
        self.offset = 0;
    }

    /// Returns a slice of the valid data in the buffer.
    pub fn data(&self) -> &[u8] {
        &self.data[..self.offset]
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_buffer() {
		let buffer = DirtyBuffer::new("test");
		assert_eq!(buffer.data, Vec::<u8>::new());
		assert_eq!(buffer.dirty, false);
		assert_eq!(buffer.offset, 0);
	}

	#[test]
	fn test_clear_buffer() {
		let mut buffer = DirtyBuffer::new("test");
		buffer.extend_from_slice(&[1, 2, 3]);
		buffer.clear();
		assert_eq!(buffer.data, Vec::<u8>::new());
		assert_eq!(buffer.dirty, true);
		assert_eq!(buffer.offset, 0);
	}

	#[test]
	fn test_extend_from_slice() {
		let mut buffer = DirtyBuffer::new("test");
		buffer.extend_from_slice(&[1, 2, 3]);
		assert_eq!(buffer.data, vec![1, 2, 3]);
		assert_eq!(buffer.dirty, true);
		assert_eq!(buffer.offset, 3);
	}

	#[test]
	fn test_extend_from_slice_no_change() {
		let mut buffer = DirtyBuffer::new("test");
		buffer.extend_from_slice(&[1, 2, 3]);
		buffer.reset_offset();
		buffer.dirty = false;
		buffer.extend_from_slice(&[1, 2, 3]);
		assert_eq!(buffer.data, vec![1, 2, 3]);
		assert_eq!(buffer.dirty, false);
		assert_eq!(buffer.offset, 3);
	}
}
