extern crate assimp;
extern crate glutin;
extern crate grr;
extern crate nalgebra_glm as glm;

mod camera;

use assimp::import::Importer;
use glutin::GlContext;
use std::{slice, time};

#[repr(C)]
struct VertexPos(pub [f32; 3]);

struct Geometry {
    pub base_index: u32,
    pub num_indices: u32,
    pub base_vertex: i32,
}

struct FrameTime {
    last: std::time::Instant,
}

impl FrameTime {
    pub fn new() -> Self {
        FrameTime {
            last: time::Instant::now(),
        }
    }

    pub fn update(&mut self) -> f32 {
        let now = time::Instant::now();
        let elapsed = self.last.elapsed();
        self.last = now;

        let NANOS_PER_SEC = 1_000_000_000;
        (elapsed.as_secs() * NANOS_PER_SEC + elapsed.subsec_nanos() as u64) as f32
            / NANOS_PER_SEC as f32
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("grr - PBR sample")
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

    let mut importer = Importer::new();
    importer.triangulate(true);

    let base_path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));
    let scene_name = "Cerberus_LP";
    let model_scene = importer
        .read_file(format!("{}/{}.fbx", base_path, scene_name).as_str())
        .unwrap();

    let mut num_vertices = 0;
    let mut num_indices = 0;
    for mesh in model_scene.mesh_iter() {
        num_vertices += mesh.num_vertices();
        num_indices += mesh.num_faces() * 3;
    }

    let vertex_size = std::mem::size_of::<f32>() * 3; // TODO
    let mesh_data_len = vertex_size as u64 * num_vertices as u64;
    let mesh_data = grr.create_buffer(
        mesh_data_len,
        grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
    );

    let index_size = 4; // u32
    let index_data_len = index_size * num_indices as u64;
    let index_data = grr.create_buffer(
        index_data_len,
        grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
    );

    println!("{:?}", (num_vertices, num_indices));

    let mut base_index = 0;
    let mut base_vertex = 0;

    let vertices_pos_cpu =
        grr.map_buffer::<VertexPos>(&mesh_data, 0..mesh_data_len, grr::MappingFlags::empty());
    let indices_cpu =
        grr.map_buffer::<u32>(&index_data, 0..index_data_len, grr::MappingFlags::empty());

    let geometries = model_scene
        .mesh_iter()
        .map(|mesh| {
            let num_local_indices = mesh.num_faces() as usize * 3;
            let num_local_vertices = mesh.num_vertices() as usize;

            for (i, vertex) in mesh.vertex_iter().enumerate() {
                let v = base_vertex + i as usize;
                vertices_pos_cpu[v] = VertexPos([vertex.x, vertex.y, vertex.z]);
            }

            for (i, face) in mesh.face_iter().enumerate() {
                let e = base_index + 3 * i;
                let raw_indices = unsafe { slice::from_raw_parts(face.indices, 3) };
                indices_cpu[e] = raw_indices[0];
                indices_cpu[e + 1] = raw_indices[1];
                indices_cpu[e + 2] = raw_indices[2];
            }

            let geometry = Geometry {
                base_index: base_index as _,
                num_indices: num_local_indices as _,
                base_vertex: base_vertex as _,
            };

            base_index += num_local_indices;
            base_vertex += num_local_vertices;

            geometry
        })
        .collect::<Vec<_>>();

    grr.unmap_buffer(&mesh_data);
    grr.unmap_buffer(&index_data);

    let pbr_vs = grr.create_shader(
        grr::ShaderStage::Vertex,
        include_bytes!("assets/Shaders/pbr.vs"),
    );
    let pbr_fs = grr.create_shader(
        grr::ShaderStage::Fragment,
        include_bytes!("assets/Shaders/pbr.fs"),
    );

    let pbr_pipeline = grr.create_graphics_pipeline(grr::GraphicsPipelineDesc {
        vertex_shader: &pbr_vs,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: Some(&pbr_fs),
    });

    let pbr_vertex_array = grr.create_vertex_array(&[
        grr::VertexAttributeDesc {
            location: 0,
            binding: 0,
            format: grr::VertexFormat::Xyz32Float,
            offset: 0,
        },
    ]);

    let depth_stencil_state = grr::DepthStencil {
        depth_test: true,
        depth_write: true,
        depth_compare_op: grr::Compare::LessEqual,
        stencil_test: false,
        stencil_front: grr::StencilFace::KEEP,
        stencil_back: grr::StencilFace::KEEP,
    };

    let mut camera = camera::Camera::new(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, 0.0));

    let mut running = true;
    let mut frame_time = FrameTime::new();
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(w, h) => window.resize(w, h),
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    camera.handle_event(input);
                }
                _ => (),
            },
            _ => (),
        });

        let dt = frame_time.update();
        camera.update(dt);

        let perspective = glm::perspective(w as f32 / h as f32, 70.0, 0.1, 1024.0);
        let view = camera.view();
        grr.bind_pipeline(&pbr_pipeline);
        grr.bind_vertex_array(&pbr_vertex_array);
        grr.bind_vertex_buffers(
            &pbr_vertex_array,
            0,
            &[
                grr::VertexBufferView {
                    buffer: &mesh_data,
                    offset: 0,
                    stride: (std::mem::size_of::<f32>() * 3) as _,
                    input_rate: grr::InputRate::Vertex,
                },
            ],
        );
        grr.bind_index_buffer(&pbr_vertex_array, &index_data);
        grr.bind_uniform_constants(
            &pbr_pipeline,
            0,
            &[grr::Constant::Mat4x4(perspective.into())],
        );
        grr.bind_uniform_constants(&pbr_pipeline, 1, &[grr::Constant::Mat4x4(view.into())]);
        grr.bind_depth_stencil_state(&depth_stencil_state);

        grr.set_viewport(
            0,
            &[
                grr::Viewport {
                    x: 0.0,
                    y: 0.0,
                    w: w as _,
                    h: h as _,
                    n: 0.0,
                    f: 1.0,
                },
            ],
        );
        grr.set_scissor(
            0,
            &[
                grr::Region {
                    x: 0,
                    y: 0,
                    w: w as _,
                    h: h as _,
                },
            ],
        );

        grr.clear_attachment(
            grr::Framebuffer::DEFAULT,
            grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]),
        );
        grr.clear_attachment(
            grr::Framebuffer::DEFAULT,
            grr::ClearAttachment::Depth(1.0),
        );

        for geometry in &geometries {
            grr.draw_indexed(
                grr::Primitive::Triangles,
                grr::IndexTy::U32,
                geometry.base_index..geometry.base_index + geometry.num_indices,
                0..1,
                geometry.base_vertex,
            );
        }

        window.swap_buffers().unwrap();
    }
}
