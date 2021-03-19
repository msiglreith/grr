use glutin::{event_loop::EventLoop, ContextBuilder};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let context = unsafe {
        ContextBuilder::new()
            .with_gl_debug_flag(true)
            .build_headless(&event_loop, (1.0, 1.0).into())?
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
