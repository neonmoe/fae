use clipboard::{ClipboardContext, ClipboardProvider};
use std::sync::Mutex;

lazy_static! {
    static ref CLIPBOARD: Mutex<ClipboardContext> = Mutex::new(ClipboardProvider::new().unwrap());
}

pub fn get() -> Option<String> {
    CLIPBOARD.lock().unwrap().get_contents().ok()
}
