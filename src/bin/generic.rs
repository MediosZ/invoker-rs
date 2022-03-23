use std::str;
use libloading::{Library, Symbol};
use trans::{py, eval};
use cstr::cstr;
use std::{ffi::c_void, ffi::CString, os::raw::c_int, os::raw::c_char};

#[link(name = "dl")]
extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *const c_void;
    fn dlsym(handle: *const c_void, name: *const c_char) -> *const c_void;
    fn dlclose(handle: *const c_void);
}

// had to look that one up in `dlfcn.h`
// in C, it's a #define. in Rust, it's a proper constant
pub const RTLD_LAZY: c_int = 0x00001;


// // unsafe extern fn(name: *const i32)
// macro_rules! expand {
//     ($func_name:ident, $sig:tt) => {
//         let $func_name: Symbol<$sig> = lib.get(b"$func_name")?;
//     };
// }

unsafe fn fn_ptr_cast<T, U, V>(fn_ptr: T, _template_fn: U) -> V
where
    T: Copy + 'static,
    U: FnPtrCast<V>,
{
    debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<usize>());

    U::fn_ptr_cast(fn_ptr)
}

unsafe trait FnPtrCast<U> {
    unsafe fn fn_ptr_cast<T>(fn_ptr: T) -> U;
}

macro_rules! impl_fn_cast {
    ( $($arg:ident),* ) => {
        unsafe impl<Fun, Out, $($arg),*> FnPtrCast<fn($($arg),*) -> Out> for Fun
        where
            Fun: Fn($($arg),*) -> Out,
        {
            unsafe fn fn_ptr_cast<T>(fn_ptr: T) -> fn($($arg),*) -> Out {
                ::std::mem::transmute(::std::ptr::read(&fn_ptr as *const T as *const *const T))
            }
        }
    }
}

impl_fn_cast!();
impl_fn_cast!(A);
impl_fn_cast!(A, B);
impl_fn_cast!(A, B, C);
impl_fn_cast!(A, B, C, D);
impl_fn_cast!(A, B, C, D, E);
impl_fn_cast!(A, B, C, D, E, F);
impl_fn_cast!(A, B, C, D, E, F, G);
impl_fn_cast!(A, B, C, D, E, F, G, H);
fn main() -> std::result::Result<(), Box<dyn std::error::Error>>{
    // fn call<A>(lib: &Library, name: &[u8], arg1: A) -> std::result::Result<A, Box<dyn std::error::Error>> {
    //     unsafe{ 
    //         let func: Symbol<fn(A) -> A> = lib.get(name)?; 
    //         let res = func(arg1);
    //         Ok(res)
    //     }
    // }
    use std::any;
    // fn call<A, B>(lib: &Library, name: &[u8], arg1: A, arg2: B) -> std::result::Result<Any, Box<dyn std::error::Error>> {
    //     unsafe{ 
    //         let func: Symbol<fn(A, B) -> *mut c_void> = lib.get(name)?; 
    //         let res = func(arg1, arg2);
    //         dbg!(res);
    //         Ok(Any::Int(abi_interface::metacall_value_to_int(res)))
    //     }
    // }
    fn call<R, A>(lib: *const c_void, name: &str, arg1: A) -> std::result::Result<R, Box<dyn std::error::Error>> {
        unsafe{ 
            let func_name = CString::new(name).unwrap();
            let func = unsafe { dlsym(lib, func_name.as_ptr()) };
            
            type Greet = unsafe extern "C" fn(name: *const c_char);
            use std::mem::transmute;
            let func: fn(arg1: A) -> R = unsafe { transmute(func) };
        
            // let func: Symbol<fn(A) -> A> = lib.get(name)?; 

            let res = func(arg1);
            Ok(res)
            // Ok(Any::Null)
            // Ok(Any::Int(abi_interface::metacall_value_to_int(res)))
        }
    }

    unsafe {
        let lib = Library::new("tmp/temp.so")?;
        
        // so the signature is the stubs we need
        let greet: Symbol<unsafe extern fn(name: *const c_char)> = lib.get(b"greet")?;
        greet(cstr!("Rust").as_ptr());
        let run: Symbol<fn()> = lib.get(b"run222")?;
        run();
        let sta: Symbol<*mut i32> = lib.get(b"STA")?;
        println!("STA: {:?}", sta);

        let add1: Symbol<unsafe fn(i32) -> i32> = lib.get(b"add1")?;
        println!("{}", add1(3));

        let lib_name = CString::new("tmp/temp.so").unwrap();
        let lib2 = unsafe { dlopen(lib_name.as_ptr(), RTLD_LAZY) };
        if lib2.is_null() {
            panic!("could not open library");
        }
        let res = call::<i32, _>(lib2, "add1", 13);
        println!("{:?}", res);

        // let func_name = CString::new("add1").unwrap();
        // let func = unsafe { dlsym(lib2, func_name.as_ptr()) };
        // let args = vec![Type::i32()];
        // let cif = Cif::new(args.into_iter(), Type::i32());
        // let n: i32 = unsafe { cif.call(CodePtr::from_ptr(func), &[arg(&123)]) };
        // println!("{}", n);
        dlclose(lib2);
    }
    
    extern "C" fn add(x: f64, y: &f64) -> f64 {
        return x + y;
    }
    

    // eval! { py
    //     "fn(name: i32) -> i32"
    // }

   
    Ok(())


}



