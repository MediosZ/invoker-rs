#![feature(rustc_private)]
#![feature(let_else)]
#![feature(once_cell)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_span;
use rustc_driver::{args, handle_options, diagnostics_registry};
use rustc_interface::{interface};
use rustc_session::config::{self, Input};
use rustc_session::{DiagnosticOutput};
use rustc_hash::{FxHashSet};
use rustc_span::symbol::{kw, sym, Ident};
use std::default::Default;
use std::env;
use std::path::PathBuf;
use std::process;
use std::str;

use libloading::{Library, Symbol};
use std::{os::raw::c_char};
use cstr::cstr;

fn run_compiler(
    at_args: &[String]
) -> interface::Result<()> {
    let args = args::arg_expand_all(at_args);
    let Some(matches) = handle_options(&args) else { return Ok(()) };
    let sopts = config::build_session_options(&matches);

    let config = interface::Config {
        opts: sopts,
        crate_cfg: FxHashSet::default(),
        crate_check_cfg: config::CheckCfg::default(),
        input: Input::File(PathBuf::from("./test.rs")),
        input_path: None,
        output_file: Some(PathBuf::from("mid_lib.so")),
        output_dir: None,
        file_loader: None,
        diagnostic_output: DiagnosticOutput::Default,
        lint_caps: Default::default(),
        parse_sess_created: None,
        register_lints: None,
        override_queries: None,
        make_codegen_backend: None,
        registry: diagnostics_registry(),
    };

    interface::run_compiler(config, |compiler| {
        
        let linker = compiler.enter(|queries| {
            queries.global_ctxt()?.peek_mut().enter(|tcx| {
                for item in tcx.hir().items() {
                    match item.kind {
                        rustc_hir::ItemKind::Static(_, _, _) 
                        | rustc_hir::ItemKind::Fn(_, _, _)
                        | rustc_hir::ItemKind::Struct(_, _)
                        | rustc_hir::ItemKind::Enum(_, _)
                        | rustc_hir::ItemKind::Impl(_) => {
                            let name = item.ident;
                            let defid = item.def_id;
                            // dbg!(item);
                            let ty = tcx.type_of(defid);
                            println!("{}:{:?}",name, ty);
                            let attrs = tcx.get_attrs(defid.to_def_id());
                            for attr in attrs.iter() {
                                if attr.has_name(sym::no_mangle){
                                    println!("no mangle!")
                                }
                                else if attr.has_name(sym::export_name) {
                                    if let Some(s) = attr.value_str() {
                                        if s.as_str().contains('\0') {
                                            panic!("export name contains null characters");
                                        }
                                        println!("export to: {s}");
                                    }
                                }
                            }
                            // dbg!(attrs);
                        },
                        _ => (),
                    }
                }
            });

            // must call this to prevent panic
            queries.ongoing_codegen()?;
            // this function must be called to run all passes.
            let linker = queries.linker()?;
            Ok(Some(linker))
        })?;
        if let Some(linker) = linker {
            // this is the "final phase" of the compilation
            // which create the final executable and write it to disk.
            linker.link()?
        }
        else{
            println!("unable to compile");
        }
        Ok(())
    })
}
use std::{ffi::c_void, ffi::CString, os::raw::c_int};

#[link(name = "dl")]
extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *const c_void;
    fn dlsym(handle: *const c_void, name: *const c_char) -> *const c_void;
    fn dlclose(handle: *const c_void);
}

// had to look that one up in `dlfcn.h`
// in C, it's a #define. in Rust, it's a proper constant
pub const RTLD_LAZY: c_int = 0x00001;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}


fn caller() -> std::result::Result<(), Box<dyn std::error::Error>> {
    
    let lib_name = CString::new("./mid_lib.so").unwrap();
    let lib = unsafe { dlopen(lib_name.as_ptr(), RTLD_LAZY) };
    if lib.is_null() {
        panic!("could not open library");
    }

    let greet_name = CString::new("greet").unwrap();
    let greet = unsafe { dlsym(lib, greet_name.as_ptr()) };
    
    type Greet = unsafe extern "C" fn(name: *const c_char);
    use std::mem::transmute;
    let greet: Greet = unsafe { transmute(greet) };

    let name = CString::new("fresh coffee").unwrap();
    unsafe {
        greet(name.as_ptr());
        // println!("{}", res);
    }

    let STA_NAME = CString::new("STA").unwrap();
    let sta = unsafe { dlsym(lib, STA_NAME.as_ptr()) };
    let sta: *mut i32 = unsafe { transmute(sta) };
    unsafe{
        println!("{:?}", *sta);
    }

    unsafe {
        dlclose(lib);
    }
    Ok(())

    // unsafe {
    //     let lib = Library::new("mid_lib.so")?;
    //     // so the signature is the stubs we need
    //     let greet: Symbol<unsafe extern fn(name: *const c_char)> = lib.get(b"greet")?;
    //     greet(cstr!("Rust").as_ptr());
    //     let run: Symbol<fn()> = lib.get(b"run")?;
    //     run();
    //     let sta: Symbol<i32> = lib.get(b"STA")?;
    //     println!("STA: {sta:?}");
    // }
    // Ok(())
}

fn main() {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = str::from_utf8(&out.stdout).unwrap().trim();
    let mut args = env::args_os()
        .enumerate()
        .map(|(i, arg)| {
            arg.into_string().unwrap_or_else(|arg| {
                format!("argument {} is not valid Unicode: {:?}", i, arg)
            })
        })
        .collect::<Vec<_>>();
    args.push("--sysroot".into());
    args.push(sysroot.into());
    args.push("--crate-type=cdylib".into());
    args.push("-C".into());
    args.push("link-dead-code".into());

    if let Err(_) = run_compiler(&args){
        process::exit(1);
    }

    println!("RUN!");

    if let Err(e) = caller() {
        println!("{:?}", e);
    }
}
