use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

struct App {}

impl pge::App for App {
	fn on_create(&mut self, state: &mut pge::State) {}
	fn on_keyboard_input(&mut self, key: pge::KeyboardKey, action: pge::KeyAction, state: &mut pge::State) {}
	fn on_mouse_input(&mut self, event: pge::MouseEvent, state: &mut pge::State) {}
	fn on_process(&mut self, state: &mut pge::State, delta: f32) {
		// on_proces_cb.call(delta);
	}
	fn on_phycis_update(&mut self, state: &mut pge::State, delta: f32) {}
}


pub enum Event {}

static tx: Mutex<Option<mpsc::Sender<Event>>> = Mutex::new(None);

type Callback = extern "C" fn(result: i32);

#[no_mangle]
pub extern "C" fn pge_window_create(title: *const u8, width: u32, height: u32, cb: Callback) {
	// let tx = tx.lock().unwrap();
	// tx.send(Event::WindowCreated).unwrap();
}

#[no_mangle]
pub extern "C" fn pge_set_node_translation(node_id: u32, x: f32, y: f32, z: f32) {
	
}

#[no_mangle]
pub extern "C" fn pge_get_node_translation(node_id: u32) {
	 
}

#[no_mangle]
pub extern "C" fn pge_set_node_scale(node_id: u32, x: f32, y: f32, z: f32) {
	
}

#[no_mangle]
pub extern "C" fn pge_set_node_rotation(node_id: u32, x: f32, y: f32, z: f32, w: f32) {
	
}

#[no_mangle]
pub extern "C" fn register_callback(cb: Callback) {
    thread::spawn(move || {
		let app = App {};
		pge::run(app);
	});

	thread::spawn(move || {});
}