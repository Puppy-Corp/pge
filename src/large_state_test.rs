#[allow(soft_unstable)]


use std::thread::sleep;
use std::time::Duration;

use pge::Scene;
use pge::State;
use pge::load_gltf;

// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	#[bench]
// 	fn bench_insert() {
// 		let mut state = State::default();
// 		let size = 500_000;
// 		state.scenes.reserve(size);
// 		b.iter(|| {
// 			for i in 0..size {
// 				let scene = Scene::new();
// 				state.scenes.insert(scene);
// 				load_gltf("../assets/orkki.glb", &mut state);
// 			}
// 		});
// 	}
// }

fn main() {
	let mut state = State::default();
	let size = 5_000_000;
	state.scenes.reserve(size);
	let timer = std::time::Instant::now();
	for i in 0..size {
		let scene = Scene::new();
		state.scenes.insert(scene);
		// load_gltf("../assets/orkki.glb", &mut state);
	}
	println!("Inserted in {:?}", timer.elapsed());
	println!("scenes len: {}", state.scenes.len());
	println!("state size: {} mb", state.mem_size() / 1024 / 1024);

	let timer = std::time::Instant::now();
	let mut c = 0;
	for _ in &state.scenes {
		c += 1;
	}
	println!("Iterated in {:?}", timer.elapsed());


	let timer = std::time::Instant::now();
	let cloned = state.clone();
	println!("Cloned in {:?}", timer.elapsed());


	// let big_lock = std::sync::RwLock::new(state);

	// println!("## RWLOCK test");
	// let timer = std::time::Instant::now();
	// for _ in 0..1_000_000 {
	// 	let read = big_lock.read().unwrap();
	// }
	// let elapsed = timer.elapsed();
	// let per_lock = elapsed / 1_000_000;
	// println!("Read lock took {:?}", elapsed);
	// println!("Per lock: {:?}", per_lock);

	// let timer = std::time::Instant::now();
	// for _ in 0..1_000_000 {
	// 	let mut write = big_lock.write().unwrap();
	// }
	// let elapsed = timer.elapsed();
	// let per_lock = elapsed / 1_000_000;
	// println!("Write lock took {:?}", elapsed);
	// println!("Per lock: {:?}", per_lock);

	// println!("## MUTEX test");
	// let big_lock = std::sync::Mutex::new(cloned);
	// let timer = std::time::Instant::now();
	// for _ in 0..1_000_000 {
	// 	let read = big_lock.lock().unwrap();
	// }
	// let elapsed = timer.elapsed();
	// let per_lock = elapsed / 1_000_000;
	// println!("Read lock took {:?}", elapsed);
	// println!("Per lock: {:?}", per_lock);
}