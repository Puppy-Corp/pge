use android_activity::AndroidApp;

#[no_mangle]
fn android_main(app: AndroidApp) {
	println!("Hello, Android!");
}