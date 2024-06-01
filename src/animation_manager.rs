use tokio::time::Instant;

pub struct AnimationManager {
	timer: Instant
}

impl AnimationManager {
	pub fn new() -> Self {
		Self {
			timer: Instant::now()
		}
	}
}