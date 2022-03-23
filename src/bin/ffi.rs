extern "C" fn add(x: f64, y: &f64) -> f64 {
    return x + y;
}
use std::{ffi::c_void, ffi::CString, os::raw::c_int, os::raw::c_char};
pub const RTLD_LAZY: c_int = 0x00001;
#[link(name = "dl")]
extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *const c_void;
    fn dlsym(handle: *const c_void, name: *const c_char) -> *const c_void;
    fn dlclose(handle: *const c_void);
}


// use libffi::middle::*;
use libffi::high::call::*;
fn main() {
    // let args = vec![Type::f64(), Type::pointer()];
    // let cif = Cif::new(args.into_iter(), Type::f64());
    
    // let n = unsafe { cif.call(CodePtr(add as *mut _), &[arg(&5f64), arg(&&6f64)]) };
    // assert_eq!(11f64, n);


    let lib_name = CString::new("tmp/temp.so").unwrap();
    let lib2 = unsafe { dlopen(lib_name.as_ptr(), RTLD_LAZY) };
    if lib2.is_null() {
        panic!("could not open library");
    }
    let func_name = CString::new("add1").unwrap();
    let func = unsafe { dlsym(lib2, func_name.as_ptr()) };
    // let args = vec![Type::i32()];
    // let cif = Cif::new(args.into_iter(), Type::i32());
    // let n = unsafe { cif.call::<i32>(CodePtr::from_ptr(func), &[arg(&123)]) };
    // println!("{}", n);
    let ag = 123;
    let mut args: Vec<Arg> = vec![];
    args.push(arg(&ag));
    let result = unsafe {
        call::<i32>(CodePtr::from_ptr(func), &args)
    };
    println!("!!! {}", result);
}
