
#[derive(Debug)]
pub struct ChangePrinter {
    prev: Vec<String>,
}

impl ChangePrinter {
    pub fn new() -> Self {
        Self {
            prev: Vec::new(),
        }
    }

    pub fn print(&mut self, slot: u8, new: String) {
        if self.prev.len() <= slot as usize {
			log::info!("{}", new);
			self.prev.push(new);
		} else {
			if self.prev[slot as usize] != new {
				log::info!("{}", new);
				self.prev[slot as usize] = new;
			}
		}
    }
}