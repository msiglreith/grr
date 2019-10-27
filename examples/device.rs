fn main() -> Result<(), Box<dyn std::error::Error>> {
    let events_loop = glutin::EventsLoop::new();
    let context = unsafe {
        glutin::ContextBuilder::new()
            .with_gl_debug_flag(true)
            .build_headless(&events_loop, (1.0, 1.0).into())?
            .make_current()
            .unwrap()
    };

    unsafe {
        let grr = grr::Device::new(
            |symbol| context.get_proc_address(symbol) as *const _,
            grr::Debug::Disable,
        );

        println!("{:#?}", grr.limits());
    }

    Ok(())
}
