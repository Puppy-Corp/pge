mod lib;

fn main() {
	simple_logger::init_with_level(log::Level::Info).unwrap();
	pge::run(FpsShooter::new());
}