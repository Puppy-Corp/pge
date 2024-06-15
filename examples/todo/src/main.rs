use pge::*;

struct TodoItem {
	text: String,
	completed: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Engine::new(|mut handle| async move {
		let mut window = Window::new();
		window.title = "Todo app".to_string();

		let mut todo_items: Vec<TodoItem> = Vec::new();
		
		let root = vstack()
			.add(text("Todo app"))
			.add(list().add_many(
				todo_items.iter().map(|item| text(&item.text)).collect()
			));

		window.render(root);

	}).run().await?;
	Ok(())
}
