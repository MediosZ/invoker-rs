
extern "C" fn hypot(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

use libffi::high::call::*;
use std::any::{Any, TypeId};
fn main() {

    if true {
        let result = unsafe {
            call::<f32>(CodePtr(hypot as *mut _), &[arg(&3f32), arg(&4f32)])
        };
        assert!(TypeId::of::<f32>() == result.type_id());
        println!("{}", result);
    }

}
