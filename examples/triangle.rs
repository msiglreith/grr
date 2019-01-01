extern crate glutin;
extern crate grr;

use glutin::GlContext;

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

fn main() -> grr::Result<()> {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true)
        .with_gl_debug_flag(true);

    let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    let (w, h) = window.get_inner_size().unwrap();

    unsafe {
        window.make_current().unwrap();
    }

    let grr = grr::Device::new(
        |symbol| window.get_proc_address(symbol) as *const _,
        grr::Debug::Enable {
            callback: |_, _, _, _, msg| {
                println!("{:?}", msg);
            },
            flags: grr::DebugReport::FULL,
        },
    );

    let vs = grr.create_shader(grr::ShaderStage::Vertex, VERTEX_SRC.as_bytes())?;
    let fs = grr.create_shader(grr::ShaderStage::Fragment, FRAGMENT_SRC.as_bytes())?;

    let pipeline = grr.create_graphics_pipeline(grr::GraphicsPipelineDesc {
        vertex_shader: &vs,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: Some(&fs),
    })?;

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
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(w, h) => window.resize(w, h),
                _ => (),
            },
            _ => (),
        });

        grr.bind_pipeline(&pipeline);
        grr.bind_vertex_array(&vertex_array);
        grr.bind_vertex_buffers(
            &vertex_array,
            0,
            &[grr::VertexBufferView {
                buffer: &triangle_data,
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

        window.swap_buffers().unwrap();
    }

    grr.delete_shaders(&[vs, fs]);
    grr.delete_pipeline(pipeline);
    grr.delete_buffer(triangle_data);
    grr.delete_vertex_array(vertex_array);

    Ok(())
}
