use std::str;
use std::env;
use std::process;
use invoker::compiler::{ComplingResult, run_compiler};

fn main()  -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("testing!");

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
    let mut result = ComplingResult::default();
    if let Err(_) = run_compiler("test.rs",
    "tmp/temp.so",
    &mut result,
    &args){
        process::exit(1);
    }
    println!("result: {:?}", result);
    Ok(())
}