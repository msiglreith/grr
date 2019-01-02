extern crate glutin;
extern crate grr;

use glutin::GlContext;

fn main() -> grr::Result<()> {
    let events_loop = glutin::EventsLoop::new();
    let context = glutin::Context::new(
        &events_loop,
        glutin::ContextBuilder::new().with_gl_debug_flag(true),
        false,
    )
    .unwrap();

    unsafe {
        context.make_current().unwrap();
    }

    let grr = grr::Device::new(
        |symbol| context.get_proc_address(symbol) as *const _,
        grr::Debug::Disable,
    );

    println!("{:#?}", grr.limits());

    Ok(())
}
