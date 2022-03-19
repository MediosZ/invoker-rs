use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

#[no_mangle]
static STA:i32 = 123;

#[export_name="run222"]
pub extern "C" fn run(){
    println!("Hello from compiler3!");
}

#[no_mangle]
pub unsafe fn greet(name: *const c_char) {
    let cstr = CStr::from_ptr(name);
    let name = cstr.to_str().unwrap();
    println!("Hello, {}!", name);
}
#[no_mangle]
pub fn create_lib(){
    println!("Hello from compiler3!");
}

struct Library {
    handle: *const c_void,
}

impl Library {
    pub fn test() {
        println!("test");
    }
}