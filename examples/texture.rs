use glutin::dpi::LogicalSize;

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

fn main() -> grr::Result<()> {
    unsafe {
        let mut events_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new()
            .with_title("grr - Texture")
            .with_dimensions(LogicalSize {
                width: 1024.0,
                height: 768.0,
            });
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_srgb(true)
            .with_gl_debug_flag(true)
            .build_windowed(wb, &events_loop)
            .unwrap()
            .make_current()
            .unwrap();

        let LogicalSize {
            width: mut w,
            height: mut h,
        } = window.window().get_inner_size().unwrap();

        let grr = grr::Device::new(
            |symbol| window.get_proc_address(symbol) as *const _,
            grr::Debug::Enable {
                callback: |_, _, _, _, msg| {
                    println!("{:?}", msg);
                },
                flags: grr::DebugReport::FULL,
            },
        );

        let vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            VERTEX_SRC.as_bytes(),
            grr::ShaderFlags::VERBOSE,
        )?;
        let fs = grr.create_shader(
            grr::ShaderStage::Fragment,
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
                format: grr::VertexFormat::Xy32Float,
                offset: (2 * std::mem::size_of::<f32>()) as _,
            },
        ])?;

        let vertex_buffer =
            grr.create_buffer_from_host(grr::as_u8_slice(&VERTICES), grr::MemoryFlags::empty())?;
        let index_buffer =
            grr.create_buffer_from_host(grr::as_u8_slice(&INDICES), grr::MemoryFlags::empty())?;

        let img = image::open(&Path::new("info/grr_logo.png"))
            .unwrap()
            .to_rgba();
        let img_width = img.width();
        let img_height = img.height();
        let img_data = img.into_raw();

        let (texture, texture_view) = grr.create_image_and_view(
            grr::ImageType::D2 {
                width: img_width,
                height: img_height,
                layers: 1,
                samples: 1,
            },
            grr::Format::R8G8B8A8_SRGB,
            1,
        )?;
        grr.object_name(texture, "grr logo");

        grr.copy_host_to_image(
            &img_data,
            texture,
            grr::HostImageCopy {
                host_layout: grr::MemoryLayout {
                    base_format: grr::BaseFormat::RGBA,
                    format_layout: grr::FormatLayout::U8,
                    row_length: img_width,
                    image_height: img_height,
                    alignment: 4,
                },
                image_subresource: grr::SubresourceLayers {
                    level: 0,
                    layers: 0..1,
                },
                image_offset: grr::Offset { x: 0, y: 0, z: 0 },
                image_extent: grr::Extent {
                    width: img_width,
                    height: img_height,
                    depth: 1,
                },
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
        })?;

        let color_blend = grr::ColorBlend {
            attachments: vec![grr::ColorBlendAttachment {
                blend_enable: true,
                color: grr::BlendChannel {
                    src_factor: grr::BlendFactor::SrcAlpha,
                    dst_factor: grr::BlendFactor::OneMinusSrcAlpha,
                    blend_op: grr::BlendOp::Add,
                },
                alpha: grr::BlendChannel {
                    src_factor: grr::BlendFactor::SrcAlpha,
                    dst_factor: grr::BlendFactor::OneMinusSrcAlpha,
                    blend_op: grr::BlendOp::Add,
                },
            }],
        };

        let mut running = true;
        while running {
            events_loop.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(size) => {
                        w = size.width;
                        h = size.height;
                        let dpi_factor = window.window().get_hidpi_factor();
                        window.resize(size.to_physical(dpi_factor));
                    }
                    _ => (),
                },
                _ => (),
            });

            grr.bind_pipeline(pipeline);
            grr.bind_vertex_array(vertex_array);
            grr.bind_color_blend_state(&color_blend);

            grr.bind_image_views(3, &[texture_view]);
            grr.bind_samplers(3, &[sampler]);

            grr.bind_index_buffer(vertex_array, index_buffer);
            grr.bind_vertex_buffers(
                vertex_array,
                0,
                &[grr::VertexBufferView {
                    buffer: vertex_buffer,
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
                grr::ClearAttachment::ColorFloat(0, [0.9, 0.9, 0.9, 1.0]),
            );
            grr.draw_indexed(grr::Primitive::Triangles, grr::IndexTy::U16, 0..6, 0..1, 0);

            window.swap_buffers().unwrap();
        }

        grr.delete_shaders(&[vs, fs]);
        grr.delete_pipeline(pipeline);
        grr.delete_sampler(sampler);
        grr.delete_image_view(texture_view);
        grr.delete_image(texture);
        grr.delete_vertex_array(vertex_array);
        grr.delete_buffers(&[vertex_buffer, index_buffer]);
    }

    Ok(())
}
