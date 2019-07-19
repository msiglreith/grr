extern crate gl_generator;

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dst = Path::new(&out_dir);
    let mut file = File::create(&dst.join("gl_bindings.rs")).unwrap();

    Registry::new(
        Api::Gl,
        (4, 6),
        Profile::Core,
        Fallbacks::All,
        &["GL_NV_mesh_shader"],
    )
    .write_bindings(StructGenerator, &mut file)
    .unwrap();
}
