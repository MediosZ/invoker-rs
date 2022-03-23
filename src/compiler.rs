
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_span;
extern crate rustc_middle;
extern crate rustc_ast;
use rustc_driver::{args, handle_options, diagnostics_registry};
use rustc_interface::{interface};
use rustc_session::config::{self, Input};
use rustc_session::{DiagnosticOutput};
use rustc_hash::{FxHashSet};
use rustc_span::symbol::{sym};
use rustc_hir::{FnSig, TyKind, QPath, def::Res, PrimTy, FnRetTy};
use rustc_middle::ty;
use rustc_ast::{IntTy, FloatTy};
use std::default::Default;
use std::path::{PathBuf};
use std::str;
use std::env;
use std::process;
use super::Any;
use std::collections::HashMap;


#[derive(Debug, Default)]
pub struct Varibale{
    pub ty: Any
}

#[derive(Debug, Default)]
pub struct Function{
    pub args: Vec<Any>,
    pub ret: Any,
    pub sig: String 
}

fn kind_to_any(kind: &ty::TyKind) -> Any {
    match kind {
        ty::TyKind::Int(t) => {
            match t {
                ty::IntTy::I16 => Any::Short(0),
                ty::IntTy::I32 => Any::Int(0),
                ty::IntTy::I64 => Any::Long(0),
                _ => Any::Null
            }
        },
        _ => Any::Null,
    }
}

fn str_to_any(kind: &str) -> Any {
    match kind {
        "i32" => Any::Int(0),
        "i16" => Any::Short(0),
        "i64" => Any::Long(0),
        "f32" => Any::Float(0.0),
        "f64" => Any::Double(0.0),
        "bool" => Any::Bool(true),
        _ => Any::Null
    }
}
// ->\s?([a-z0-9]+)\s?{([a-z0-9_]+)}

// fn hir_kind_to_any(kind: &TyKind) -> Any {
//     match kind {
//         TyKind::Int(t) => {
//             match t {
//                 ty::IntTy::I16 => Any::Short(0),
//                 ty::IntTy::I32 => Any::Int(0),
//                 ty::IntTy::I64 => Any::Long(0),
//                 _ => Any::Null
//             }
//         },
//         _ => Any::Null,
//     }
// }

fn inspect(a: i32) {

}


#[derive(Default, Debug)]
pub struct CompilingResult{
    pub variables: HashMap<String, Varibale>,
    pub functions: HashMap<String, Function>,
    pub output: String,
}

pub fn compile(input_file: &str,
    output_file: &str) -> Option<CompilingResult>{
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
        let mut result = CompilingResult::default();
        if let Err(_) = run_compiler(input_file,
            output_file,
            &mut result,
            &args){
            // process::exit(1);
            return None;
        }
        Some(result)
    }

pub fn run_compiler(
    input_file: &str,
    output_file: &str,
    result: &mut CompilingResult,
    at_args: &[String]
) -> interface::Result<()> {
    let args = args::arg_expand_all(at_args);
    let Some(matches) = handle_options(&args) else { return Ok(()) };
    let sopts = config::build_session_options(&matches);

    let config = interface::Config {
        opts: sopts,
        crate_cfg: FxHashSet::default(),
        crate_check_cfg: config::CheckCfg::default(),
        input: Input::File(PathBuf::from(input_file)),
        input_path: None,
        output_file: Some(PathBuf::from(output_file)),
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
    result.output = output_file.to_string();

    interface::run_compiler(config, |compiler| {
        let linker = compiler.enter(|queries| {
            queries.global_ctxt()?.peek_mut().enter(|tcx| {
                for item in tcx.hir().items() {
                    match &item.kind {
                        rustc_hir::ItemKind::Static(_, _, _) => {
                            let name = item.ident;
                            let defid = item.def_id;
                            // dbg!(item);
                            let ty = tcx.type_of(defid);
                            // dbg!(ty);
                            // println!("{}:{:?}",name, ty);
                            let attrs = tcx.get_attrs(defid.to_def_id());
                            let mut no_mangle = false;
                            let mut export_name:Option<String> = None;
                            for attr in attrs.iter() {
                                if attr.has_name(sym::no_mangle){
                                    // println!("no mangle!")
                                    no_mangle = true;
                                }
                                else if attr.has_name(sym::export_name) {
                                    if let Some(s) = attr.value_str() {
                                        if s.as_str().contains('\0') {
                                            panic!("export name contains null characters");
                                        }
                                        // println!("export to: {s}");
                                        export_name = Some(String::from(s.as_str()));
                                    }
                                }
                            }
                            if let Some(ename) = export_name {
                                result.variables.insert(ename, Varibale {ty: str_to_any(&ty.to_string()) });
                            }
                            else{
                                if no_mangle {
                                    result.variables.insert(name.to_string(), Varibale {ty: str_to_any(&ty.to_string())});
                                }
                            }
                            // dbg!(attrs);
                        },
                        rustc_hir::ItemKind::Fn(sig2, _, _) => {
                            let mut function = Function::default();
                            let name = item.ident;
                            let defid = item.def_id;
                            // dbg!(item);
                            // dbg!(ty);
                            // dbg!(sig2);
                            // inspect(sig);
                            // println!("{}:{}",name, ty.to_string());

                            // println!("func: {}", sig);
                            // parse attributes
                            let attrs = tcx.get_attrs(defid.to_def_id());
                            let mut no_mangle = false;
                            let mut export_name:Option<String> = None;
                            let mut export = false;
                            let mut fname = String::new();
                            for attr in attrs.iter() {
                                if attr.has_name(sym::no_mangle){
                                    // println!("no mangle!")
                                    no_mangle = true;
                                }
                                else if attr.has_name(sym::export_name) {
                                    if let Some(s) = attr.value_str() {
                                        if s.as_str().contains('\0') {
                                            panic!("export name contains null characters");
                                        }
                                        // println!("export to: {s}");
                                        export_name = Some(String::from(s.as_str()));
                                    }
                                }
                            }
                            if let Some(ename) = export_name {
                                export = true;
                                fname = ename;
                            }
                            else{
                                if no_mangle {
                                    export = true;
                                    fname = name.to_string();
                                }
                            }
                            if !export {
                                continue
                            }
                            let ty = tcx.type_of(defid);
                            let sig = ty.to_string();
                            function.sig = sig;
                            // parse input and output
                            for arg in sig2.decl.inputs {
                                match &arg.kind {
                                    TyKind::Path(path) => {
                                        // let p = path.last_segment_span();
                                        // dbg!(p);
                                        if let QPath::Resolved(_, rpath) = path {
                                            // dbg!(rpath.res);
                                            if let Res::PrimTy(typ) = rpath.res {
                                                // dbg!(typ);
                                                match typ {
                                                    PrimTy::Int(ty) => {
                                                        match ty {
                                                            IntTy::I16 => function.args.push(Any::Short(0)),
                                                            IntTy::I32 => function.args.push(Any::Int(0)),
                                                            IntTy::I64 => function.args.push(Any::Long(0)),
                                                            _ => function.args.push(Any::Null),
                                                        }
                                                        // function.args.push(Any::Int(0));
                                                    },
                                                    PrimTy::Float(ty) => {
                                                        match ty {
                                                            FloatTy::F32 => function.args.push(Any::Float(0.0)),
                                                            FloatTy::F64 => function.args.push(Any::Double(0.0)),
                                                        }
                                                        // function.args.push(Any::Float(0.0));
                                                    },
                                                    PrimTy::Bool => {
                                                        function.args.push(Any::Bool(true));
                                                    },
                                                    _ => {
                                                        
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                                // dbg!(&arg.kind);
                                // function.args.push()
                            }

                            match sig2.decl.output {
                                FnRetTy::DefaultReturn(_) => function.ret = Any::Null,
                                FnRetTy::Return(ty) => {
                                    match &ty.kind {
                                        TyKind::Path(path) => {
                                            // let p = path.last_segment_span();
                                            // dbg!(p);
                                            if let QPath::Resolved(_, rpath) = path {
                                                // dbg!(rpath.res);
                                                if let Res::PrimTy(typ) = rpath.res {
                                                    // dbg!(typ);
                                                    match typ {
                                                        PrimTy::Int(ty) => {
                                                            match ty {
                                                                IntTy::I16 => function.ret = Any::Short(0),
                                                                IntTy::I32 => function.ret = Any::Int(0),
                                                                IntTy::I64 => function.ret = Any::Long(0),
                                                                _ => function.ret = Any::Null,
                                                            }
                                                            // function.args.push(Any::Int(0));
                                                        },
                                                        PrimTy::Float(ty) => {
                                                            match ty {
                                                                FloatTy::F32 => function.ret = Any::Float(0.0),
                                                                FloatTy::F64 => function.ret = Any::Double(0.0),
                                                            }
                                                            // function.args.push(Any::Float(0.0));
                                                        },
                                                        PrimTy::Bool => {
                                                            function.ret = Any::Bool(true);
                                                        },
                                                        _ => {
                                                            
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            
                            result.functions.insert(fname, function);
                        },
                        rustc_hir::ItemKind::Struct(_, _)
                        | rustc_hir::ItemKind::Enum(_, _)
                        | rustc_hir::ItemKind::Impl(_) => {
                            
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

#[cfg(test)]
mod tests{
    use std::str;
    use std::env;
    use std::process;
    use crate::compiler::{CompilingResult, run_compiler};

    #[test]
    fn test_main() {
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
        let mut result = CompilingResult::default();
        if let Err(_) = run_compiler("test.rs",
        "tmp/temp.so",
        &mut result,
        &args){
            process::exit(1);
        }
        println!("result: {:?}", result);

        assert_eq!(1, 1);
    }


}
