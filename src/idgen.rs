use std::sync::atomic::AtomicUsize;

static ID: AtomicUsize = AtomicUsize::new(0);
pub fn gen_id() -> usize {
	ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}