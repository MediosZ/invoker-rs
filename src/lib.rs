#![feature(rustc_private)]
#![feature(let_else)]
#![feature(once_cell)]
use std::{ffi::c_void, ffi::CString};
pub const RTLD_LAZY: c_int = 0x00001;
#[link(name = "dl")]
extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *const c_void;
    fn dlsym(handle: *const c_void, name: *const c_char) -> *const c_void;
    fn dlclose(handle: *const c_void);
}
use std::os::raw::{c_char, c_double, c_float, c_int, c_long, c_short};

use libffi::high::call::{arg, call, CodePtr, Arg};
use std::path::Path;

/// Enum of all possible Metacall types to allow for safe conversion between them and c_types
#[derive(Debug)]
pub enum Any {
    Null,              // from c_null
    Short(i16),        // from c_short
    Int(i32),          // from c_int
    Long(i64),         // from c_long
    Float(f32),        // from c_float
    Double(f64),       // from c_double
    Bool(bool),        // from c_bool
    Char(char),        // from c_char
    Str(String),       // from *const u8 (null terminated)
    Array(Vec<Any>),   // from *mut *mut c_void
    Buffer(Vec<u8>),   // from *const u8 (non-null terminated) (raw binary data)
    Pointer(Box<Any>), // from *mut c_void
    Function(Box<fn(Any) -> Any>), // from a C function pointer
                       // METACALL_FUTURE
}

impl From<String> for Any {
    fn from(val: String) -> Self {
        match val {
            _ => todo!()
        }
    }
}

impl From<c_short> for Any {
    fn from(val: c_short) -> Self {
        Any::Short(val)
    }
}
impl From<c_int> for Any {
    fn from(val: c_int) -> Self {
        Any::Int(val)
    }
}
impl From<c_long> for Any {
    fn from(val: c_long) -> Self {
        Any::Long(val)
    }
}
impl From<c_char> for Any {
    fn from(val: c_char) -> Self {
        Any::Char(val as u8 as char)
    }
}
impl From<bool> for Any {
    fn from(val: bool) -> Self {
        Any::Bool(val)
    }
}
impl From<c_float> for Any {
    fn from(val: c_float) -> Self {
        Any::Float(val)
    }
}
impl From<c_double> for Any {
    fn from(val: c_double) -> Self {
        Any::Double(val)
    }
}
impl Default for Any {
    fn default() -> Self {
        Any::Null
    }
}

pub mod compiler;
use compiler::{CompilingResult, compile};

#[derive(Default)]
pub struct Invoker{
    lib_path: String,
    lib: Option<*const c_void>,
    compile_result: Option<CompilingResult>
}

impl Drop for Invoker {
    fn drop(&mut self){
        if let Some(lib) = self.lib {
            println!("close {}", self.lib_path);
            unsafe {
                dlclose(lib);
            }
        }
    }
}

// fn str_to_ffiarg(value: &str, ty: Any) -> Arg {
//     match ty {
//         Any::Int(_) => arg(&str::parse::<i32>(value).expect("unable to convert arg")),
//         Any::Short(_) => arg(&str::parse::<i16>(value).expect("unable to convert arg")),
//         Any::Long(_) => arg(&str::parse::<i64>(value).expect("unable to convert arg")),
//         Any::Float(_) => arg(&str::parse::<f32>(value).expect("unable to convert arg")),
//         Any::Double(_) => arg(&str::parse::<f64>(value).expect("unable to convert arg")),
//         _ => todo!()
//     }
// }

impl Invoker{
    pub fn load(&mut self, s: &str){
        println!("load {s}");
        if !Path::new(s).exists() {
            println!("{} doesn't exist", s);
            return;
        }
        if self.lib.is_some() {
            println!("Already load a lib, please unload first");
            return;
        }
        self.lib_path = "tmp/temp.so".to_string();
        if let Some(res) = compile(s, self.lib_path.as_str()) {
            self.compile_result = Some(res);
        }
        let lib_name = CString::new(self.lib_path.as_str()).unwrap();
        let lib = unsafe { dlopen(lib_name.as_ptr(), RTLD_LAZY) };
        if lib.is_null() {
            panic!("could not open library");
        }
        self.lib = Some(lib);
        println!("successfully load {}", s);
    }

    pub fn inspect(&self){
        println!("inspect!");
        if let Some(res) = &self.compile_result {
            println!("Variables: ");
            for var in &res.variables {
                println!("{:?}", var);
            }
            println!("Functions: ");
            for func in &res.functions {
                println!("{:?}", func);
            }
        }
        else{
            println!("nothing here");
        }

    }
    
    pub fn call(&self, name: &str, args: Vec<&str>){
        println!("call {name}");
        // println!("args: {:?}", args);
        if let Some(lib) = self.lib {
            let func_name = CString::new(name).unwrap();
            let func = unsafe { dlsym(lib, func_name.as_ptr()) };
            if func.is_null() {
                println!("unable to locate func: {}", name);
                return;
            }
            // parse args
            if let Some(res) = self.compile_result.as_ref() {
                if let Some(function) = res.functions.get(&name.to_string()) {
                    if function.args.len() != args.len() {
                        println!("number of arg mismatch");
                        return;
                    }
                    let mut anyargs: Vec<Any> = vec![];
                    
                    for idx in 0..args.len() {
                        let arg_str = args[idx].to_owned();
                        match function.args[idx] {
                            Any::Int(_) => anyargs.push(Any::from(str::parse::<i32>(arg_str.as_str()).expect("unable to convert arg"))),
                            Any::Short(_) => anyargs.push(Any::from(str::parse::<i16>(arg_str.as_str()).expect("unable to convert arg"))),
                            Any::Long(_) => anyargs.push(Any::from(str::parse::<i64>(arg_str.as_str()).expect("unable to convert arg"))),
                            Any::Float(_) => anyargs.push(Any::from(str::parse::<f32>(arg_str.as_str()).expect("unable to convert arg"))),
                            Any::Double(_) => anyargs.push(Any::from(str::parse::<f64>(arg_str.as_str()).expect("unable to convert arg"))),
                            _ => todo!()
                        };
                        // ffiargs.push(ffiarg);
                    }
                    let ffiargs: Vec<Arg> = anyargs.iter().map(|anyarg| {
                        match anyarg {
                            Any::Int(a) => arg(a),
                            Any::Short(a) => arg(a),
                            Any::Long(a) => arg(a),
                            Any::Float(a) => arg(a),
                            Any::Double(a) => arg(a),
                            _ => todo!()
                        }
                    }).collect();
                    let result: Any = match function.ret {
                        Any::Int(_) => unsafe {Any::from(call::<i32>(CodePtr::from_ptr(func), &ffiargs))},
                        Any::Short(_) => unsafe {Any::from(call::<i16>(CodePtr::from_ptr(func), &ffiargs))},
                        Any::Long(_) => unsafe {Any::from(call::<i64>(CodePtr::from_ptr(func), &ffiargs))},
                        Any::Float(_) => unsafe {Any::from(call::<f32>(CodePtr::from_ptr(func), &ffiargs))},
                        Any::Double(_) => unsafe {Any::from(call::<f64>(CodePtr::from_ptr(func), &ffiargs))},
                        Any::Null => {
                            unsafe {
                                call::<()>(CodePtr::from_ptr(func), &ffiargs);
                            }
                            Any::Null
                        },
                        _ => todo!()
                    };
                    // let result = unsafe {call::<i32>(CodePtr::from_ptr(func), &ffiargs)};
                    println!("{:?}", result);
                }
            }
        }
        else{
            println!("Please load a library first.");
        }

    }

    pub fn unload(&mut self) {
        if self.compile_result.is_some() {
            self.compile_result = None;
        }
        else{
            println!("no compile result to unload");
        }
        if self.lib.is_some() {
            unsafe{ dlclose(self.lib.unwrap()); }
            self.lib = None;
        }
        else{
            println!("no lib to unload");
        }

    }
}