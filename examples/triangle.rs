use glutin::dpi::LogicalSize;
use std::error::Error;

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

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut events_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new()
            .with_title("grr - Triangle")
            .with_dimensions(LogicalSize {
                width: 1024.0,
                height: 768.0,
            });
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_srgb(true)
            .with_gl_debug_flag(true)
            .build_windowed(wb, &events_loop)?
            .make_current()
            .unwrap();

        let LogicalSize {
            width: w,
            height: h,
        } = window.window().get_inner_size().unwrap();

        let grr = grr::Device::new(
            |symbol| window.get_proc_address(symbol) as *const _,
            grr::Debug::Enable {
                callback: |report, _, _, _, msg| {
                    println!("{:?}: {:?}", report, msg);
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

        let mut running = true;
        while running {
            events_loop.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(size) => {
                        let dpi_factor = window.window().get_hidpi_factor();
                        window.resize(size.to_physical(dpi_factor));
                    }
                    _ => (),
                },
                _ => (),
            });
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
                    w: w as _,
                    h: h as _,
                    n: 0.0,
                    f: 1.0,
                }],
            );
            grr.set_scissor(
                0,
                &[grr::Region {
                    x: 0,
                    y: 0,
                    w: w as _,
                    h: h as _,
                }],
            );

            grr.clear_attachment(
                grr::Framebuffer::DEFAULT,
                grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]),
            );
            grr.draw(grr::Primitive::Triangles, 0..3, 0..1);

            window.swap_buffers()?;
        }

        grr.delete_shaders(&[vs, fs]);
        grr.delete_pipeline(pipeline);
        grr.delete_buffer(triangle_data);
        grr.delete_vertex_array(vertex_array);
    }

    Ok(())
}
