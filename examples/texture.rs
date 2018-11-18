extern crate glutin;
extern crate grr;
extern crate image;

use glutin::GlContext;

use std::path::Path;

const VERTEX_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec2 v_pos;
    layout (location = 1) in vec2 v_uv;

    layout (location = 0) out vec2 a_uv;

    void main() {
        a_uv = v_uv;
        gl_Position = vec4(v_pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec2 a_uv;
    out vec4 f_color;

    layout (binding = 3) uniform sampler2D u_texture;

    void main() {
       f_color = texture(u_texture, a_uv);
    }
"#;

const VERTICES: [f32; 16] = [
    -0.5, -0.5, 0.0, 1.0, // bottom-left
    0.5, -0.5, 1.0, 1.0, // bottom-right
    0.5, 0.5, 1.0, 0.0, // top-right
    -0.5, 0.5, 0.0, 0.0, // top-left
];

const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true);

    let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    let (w, h) = window.get_inner_size().unwrap();

    unsafe {
        window.make_current().unwrap();
    }

    let grr = grr::Device::new(|symbol| window.get_proc_address(symbol) as *const _);

    let vs = grr.create_shader(grr::ShaderStage::Vertex, VERTEX_SRC.as_bytes());
    let fs = grr.create_shader(grr::ShaderStage::Fragment, FRAGMENT_SRC.as_bytes());

    let pipeline = grr.create_graphics_pipeline(grr::GraphicsPipelineDesc {
        vertex_shader: &vs,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: Some(&fs),
    });

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
            format: grr::VertexFormat::Xy32Float,
            offset: (2 * std::mem::size_of::<f32>()) as _,
        },
    ]);

    let vertex_buffer = {
        let len = (std::mem::size_of::<f32>() * VERTICES.len()) as u64;

        let buffer = grr.create_buffer(
            len,
            grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
        );

        let data = grr.map_buffer::<f32>(&buffer, 0..len, grr::MappingFlags::empty());
        data.clone_from_slice(&VERTICES);
        grr.unmap_buffer(&buffer);

        buffer
    };

    let index_buffer = {
        let len = (std::mem::size_of::<u16>() * INDICES.len()) as u64;

        let buffer = grr.create_buffer(
            len,
            grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
        );

        let data = grr.map_buffer::<u16>(&buffer, 0..len, grr::MappingFlags::empty());
        data.clone_from_slice(&INDICES);
        grr.unmap_buffer(&buffer);

        buffer
    };

    let img = image::open(&Path::new("info/grr_logo.png"))
        .unwrap()
        .to_rgba();
    let img_width = img.width();
    let img_height = img.height();
    let img_data = img.into_raw();

    let texture = grr.create_image(
        grr::ImageType::D2 {
            width: img_width,
            height: img_height,
            layers: 1,
            samples: 1,
        },
        grr::Format::R8G8B8A8_SRGB,
        1,
    );

    grr.copy_host_to_image(
        &texture,
        0,
        0..1,
        grr::Offset { x: 0, y: 0, z: 0 },
        grr::Extent {
            width: img_width,
            height: img_height,
            depth: 1,
        },
        &img_data,
        grr::BaseFormat::RGBA,
        grr::FormatLayout::U8,
    );

    let texture_view = grr.create_image_view(
        &texture,
        grr::ImageViewType::D2,
        grr::Format::R8G8B8A8_SRGB,
        grr::SubresourceRange {
            layers: 0..1,
            levels: 0..1,
        },
    );

    let sampler = grr.create_sampler(grr::SamplerDesc {
        min_filter: grr::Filter::Linear,
        mag_filter: grr::Filter::Linear,
        mip_map: None,
        address: (
            grr::SamplerAddress::ClampEdge,
            grr::SamplerAddress::ClampEdge,
            grr::SamplerAddress::ClampEdge,
        ),
        lod_bias: 0.0,
        lod: 0.0..10.0,
        compare: None,
        border_color: [0.0, 0.0, 0.0, 1.0],
    });

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
        grr.bind_image_views(3, &[&texture_view]);
        grr.bind_samplers(3, &[&sampler]);

        grr.bind_index_buffer(&vertex_array, &index_buffer);
        grr.bind_vertex_buffers(
            &vertex_array,
            0,
            &[grr::VertexBufferView {
                buffer: &vertex_buffer,
                offset: 0,
                stride: (std::mem::size_of::<f32>() * 4) as _,
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
        grr.draw_indexed(grr::Primitive::Triangles, grr::IndexTy::U16, 0..6, 0..1, 0);

        window.swap_buffers().unwrap();
    }
}
