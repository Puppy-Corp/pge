
pub use pgecore::*;

pub fn run(app: impl App) {

}

#[cfg(target_os = "android")]
pub fn android_main(app: AndroidApp) {
	run(app);
}

