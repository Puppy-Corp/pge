use std::time::Duration;

use pge::*;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Engine::new(|mut handle| async move {
		let mut window = Window::new();
		window.title = "BIG box".to_string();
		handle.save_window(&window);

		let mut window2 = Window::new();
		window2.title = "SMALL box".to_string();
		handle.save_window(&window2);

		sleep(Duration::from_secs(10)).await;
	}).run().await?;
	Ok(())
}
