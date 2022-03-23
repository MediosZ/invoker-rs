use std::ffi::CStr;
use std::os::raw::{c_char, c_void, c_int};

#[no_mangle]
static STA:i32 = 123;

#[export_name="run222"]
pub extern "C" fn run(){
    println!("Hello from compiler3!");
}

#[no_mangle]
pub unsafe fn add1(a: i32) -> i32 {
    a + 1
}
#[no_mangle]
pub unsafe fn add2(a: i32, b:i32) -> i32 {
    a + b
}

#[no_mangle]
pub unsafe fn greet(name: *const c_char) {
    let cstr = CStr::from_ptr(name);
    let name = cstr.to_str().unwrap();
    println!("Hello, {}!", name);
}
pub unsafe fn greet2(name: *mut c_char) {
    let cstr = CStr::from_ptr(name);
    let name = cstr.to_str().unwrap();
    println!("Hello, {}!", name);
}
pub fn create_lib(){
    println!("Hello from compiler3!");
}

// struct Library {
//     handle: *const c_void,
// }

// impl Library {
//     pub fn test() {
//         println!("test");
//     }
// }