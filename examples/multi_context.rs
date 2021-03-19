use glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use core::ffi::c_void;

const VERTEX_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec2 v_pos;
    layout (location = 1) in vec3 v_color;

    layout (location = 0) out vec3 a_color;

    void main() {
        a_color = v_color;
        gl_Position = vec4(v_pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec3 a_color;
    out vec4 f_color;

    void main() {
       f_color = vec4(a_color, 1.0);
    }
"#;

const VERTICES: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0,
];

pub struct ErasedWindowContext<W>(Option<glutin::ContextWrapper<glutin::PossiblyCurrent, W>>);

impl<W> ErasedWindowContext<W> {
    pub fn swap_buffers(&self) -> Result<(), glutin::ContextError> {
        self.0.as_ref().unwrap().swap_buffers()
    }

    pub fn resize(&self, size: glutin::dpi::PhysicalSize<u32>) {
        self.0.as_ref().unwrap().resize(size)
    }

    pub fn new(ctxt: glutin::ContextWrapper<glutin::NotCurrent, W>) -> Self {
        Self(Some(unsafe { ctxt.treat_as_current() }))
    }

    pub unsafe fn make_current(&mut self) -> Result<(), glutin::ContextError> {
        let ctxt = self.0.take().unwrap();
        let result = ctxt.make_current();
        match result {
            Ok(ctxt) => {
                self.0 = Some(ctxt);
                Ok(())
            }
            Err((ctxt, err)) => {
                self.0 = Some(ctxt.treat_as_current());
                Err(err)
            }
        }
    }

    pub fn get_proc_address(&self, addr: &str) -> *const c_void {
        self.0.as_ref().unwrap().get_proc_address(addr)
    }
}

pub struct ErasedContext(Option<glutin::Context<glutin::PossiblyCurrent>>);

impl ErasedContext {
    pub fn new(ctxt: glutin::Context<glutin::NotCurrent>) -> Self {
        Self(Some(unsafe { ctxt.treat_as_current() }))
    }

    pub unsafe fn make_current(&mut self) -> Result<(), glutin::ContextError> {
        let ctxt = self.0.take().unwrap();
        let result = ctxt.make_current();
        match result {
            Ok(ctxt) => {
                self.0 = Some(ctxt);
                Ok(())
            }
            Err((ctxt, err)) => {
                self.0 = Some(ctxt.treat_as_current());
                Err(err)
            }
        }
    }

    pub fn get_proc_address(&self, addr: &str) -> *const c_void {
        self.0.as_ref().unwrap().get_proc_address(addr)
    }
}

fn main() -> anyhow::Result<()> {
    unsafe {
        let event_loop = EventLoop::new();

        let context = glutin::ContextBuilder::new()
            .with_srgb(true)
            .with_gl_debug_flag(true)
            .build_headless(&event_loop, (1, 1).into())
            .unwrap();

        let wb = WindowBuilder::new()
            .with_title("grr - MultiContext")
            .with_inner_size(LogicalSize::new(1024.0, 768.0));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_srgb(true)
            .with_gl_debug_flag(true)
            .with_shared_lists(&context)
            .build_windowed(wb, &event_loop)
            .unwrap();

        let (present_ctxt, window) = window.split();
        let mut present_ctxt = ErasedWindowContext::new(present_ctxt);
        present_ctxt.make_current().unwrap();

        let swapchain = grr::Device::new(
            |symbol| present_ctxt.get_proc_address(symbol) as *const _,
            grr::Debug::Disable,
        );

        let present_fbo = swapchain.create_framebuffer()?;

        let mut context = ErasedContext::new(context);
        context.make_current().unwrap();

        let grr = grr::Device::new(
            |symbol| context.get_proc_address(symbol) as *const _,
            grr::Debug::Enable {
                callback: |_, _, _, _, msg| {
                    println!("{:?}", msg);
                },
                flags: grr::DebugReport::FULL,
            },
        );

        let vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            VERTEX_SRC.as_bytes(),
            grr::ShaderFlags::VERBOSE,
        )?;
        let fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            FRAGMENT_SRC.as_bytes(),
            grr::ShaderFlags::VERBOSE,
        )?;

        let pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        let vertex_array = grr.create_vertex_array(&[
            grr::VertexAttributeDesc {
                location: 0,
                binding: 0,
                format: grr::VertexFormat::Xy32Float,
                offset: 0,
            },
            grr::VertexAttributeDesc {
                location: 1,
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                offset: (2 * std::mem::size_of::<f32>()) as _,
            },
        ])?;

        let triangle_data =
            grr.create_buffer_from_host(grr::as_u8_slice(&VERTICES), grr::MemoryFlags::empty())?;

        let ctxt_fbo = grr.create_framebuffer()?;

        let size = window.inner_size();
        let present_image = grr.create_image(
            grr::ImageType::D2 {
                width: size.width as _,
                height: size.height as _,
                layers: 1,
                samples: 1,
            },
            grr::Format::R8G8B8A8_SRGB,
            1,
        )?;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::LoopDestroyed => {
                    grr.delete_shaders(&[vs, fs]);
                    grr.delete_pipeline(pipeline);
                    grr.delete_buffer(triangle_data);
                    grr.delete_vertex_array(vertex_array);
                }
                Event::RedrawRequested(_) => {
                    let size = window.inner_size();

                    context.make_current().unwrap();
                    grr.bind_pipeline(pipeline);
                    grr.bind_vertex_array(vertex_array);
                    grr.bind_vertex_buffers(
                        vertex_array,
                        0,
                        &[grr::VertexBufferView {
                            buffer: triangle_data,
                            offset: 0,
                            stride: (std::mem::size_of::<f32>() * 5) as _,
                            input_rate: grr::InputRate::Vertex,
                        }],
                    );

                    grr.set_viewport(
                        0,
                        &[grr::Viewport {
                            x: 0.0,
                            y: 0.0,
                            w: size.width as _,
                            h: size.height as _,
                            n: 0.0,
                            f: 1.0,
                        }],
                    );
                    grr.set_scissor(
                        0,
                        &[grr::Region {
                            x: 0,
                            y: 0,
                            w: size.width as _,
                            h: size.height as _,
                        }],
                    );

                    grr.bind_draw_framebuffer(ctxt_fbo);
                    grr.set_color_attachments(ctxt_fbo, &[0]);
                    grr.bind_attachments(
                        ctxt_fbo,
                        &[(
                            grr::Attachment::Color(0),
                            grr::AttachmentView::Image(present_image.as_view()),
                        )],
                    );

                    grr.clear_attachment(
                        ctxt_fbo,
                        grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]),
                    );
                    grr.draw(grr::Primitive::Triangles, 0..3, 0..1);

                    present_ctxt.make_current().unwrap();

                    swapchain.set_color_attachments(present_fbo, &[0]);
                    swapchain.bind_attachments(
                        present_fbo,
                        &[(
                            grr::Attachment::Color(0),
                            grr::AttachmentView::Image(present_image.as_view()),
                        )],
                    );

                    let screen = grr::Region {
                        x: 0,
                        y: 0,
                        w: size.width as _,
                        h: size.height as _,
                    };
                    swapchain.blit(
                        present_fbo,
                        screen,
                        grr::Framebuffer::DEFAULT,
                        screen,
                        grr::Filter::Linear,
                    );
                    present_ctxt.swap_buffers().unwrap();
                }
                _ => (),
            }
        })
    }
}
