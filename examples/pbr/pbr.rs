use nalgebra_glm as glm;

mod camera;

use assimp::import::Importer;
use raw_gl_context::{GlConfig, GlContext, Profile};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use image::Pixel;
use std::{fs, io, mem, path::Path, slice, time};

const NANOS_PER_SEC: u64 = 1_000_000_000;

#[repr(C)]
struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normals: [f32; 3],
}

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

        (elapsed.as_secs() * NANOS_PER_SEC + elapsed.subsec_nanos() as u64) as f32
            / NANOS_PER_SEC as f32
    }
}

fn max_mip_levels_2d(width: u32, height: u32) -> u32 {
    (width.max(height) as f32).log2() as u32 + 1
}

fn main() -> anyhow::Result<()> {
    unsafe {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("grr :: pbr")
            .with_inner_size(LogicalSize::new(1440.0, 700.0))
            .build(&event_loop)?;

        let context = GlContext::create(
            &window,
            GlConfig {
                version: (4, 5),
                profile: Profile::Core,
                red_bits: 8,
                blue_bits: 8,
                green_bits: 8,
                alpha_bits: 0,
                depth_bits: 24,
                stencil_bits: 8,
                samples: None,
                srgb: true,
                double_buffer: true,
                vsync: true,
            },
        )
        .unwrap();

        context.make_current();

        let grr = grr::Device::new(
            |symbol| context.get_proc_address(symbol) as *const _,
            grr::Debug::Enable {
                callback: |report, _, _, _, msg| {
                    println!("{:?}: {:?}", report, msg);
                },
                flags: grr::DebugReport::FULL,
            },
        );

        let mut importer = Importer::new();
        importer.triangulate(true);

        let base_path = format!("{}/examples/pbr/assets", env!("CARGO_MANIFEST_DIR"));
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

        let vertex_size = mem::size_of::<Vertex>();
        let mesh_data_len = vertex_size as u64 * num_vertices as u64;
        let mesh_data = grr.create_buffer(
            mesh_data_len,
            grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
        )?;

        let index_size = 4; // u32
        let index_data_len = index_size * num_indices as u64;
        let index_data = grr.create_buffer(
            index_data_len,
            grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
        )?;

        let mut base_index = 0;
        let mut base_vertex = 0;

        let vertices_cpu =
            grr.map_buffer::<Vertex>(mesh_data, 0..mesh_data_len, grr::MappingFlags::empty());
        let indices_cpu =
            grr.map_buffer::<u32>(index_data, 0..index_data_len, grr::MappingFlags::empty());

        let geometries = model_scene
            .mesh_iter()
            .map(|mesh| {
                let num_local_indices = mesh.num_faces() as usize * 3;
                let num_local_vertices = mesh.num_vertices() as usize;

                let pos_iter = mesh.vertex_iter();
                let uv_iter = mesh.texture_coords_iter(0);
                let normal_iter = mesh.normal_iter();
                for (i, ((vertex, uv), normal)) in
                    pos_iter.zip(uv_iter).zip(normal_iter).enumerate()
                {
                    let v = base_vertex + i as usize;
                    vertices_cpu[v] = Vertex {
                        pos: [vertex.x, vertex.y, vertex.z],
                        uv: [uv.x, 1.0 - uv.y],
                        normals: [normal.x, normal.y, normal.z],
                    };
                }

                for (i, face) in mesh.face_iter().enumerate() {
                    let e = base_index + 3 * i;
                    let raw_indices = slice::from_raw_parts(face.indices, 3);
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

        grr.unmap_buffer(mesh_data);
        grr.unmap_buffer(index_data);

        let load_image_rgba = |name: &str, format: grr::Format| -> anyhow::Result<grr::Image> {
            let path = format!("{}/{}", base_path, name);
            let img = image::open(&Path::new(&path)).unwrap().to_rgba();
            let img_width = img.width();
            let img_height = img.height();
            let img_data = img.into_raw();
            let num_levels = max_mip_levels_2d(img_width, img_height);

            let texture = grr.create_image(
                grr::ImageType::D2 {
                    width: img_width,
                    height: img_height,
                    layers: 1,
                    samples: 1,
                },
                format,
                num_levels,
            )?;
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
            grr.generate_mipmaps(texture);

            Ok(texture)
        };

        let albedo = load_image_rgba("Textures/Cerberus_A.tga", grr::Format::R8G8B8A8_SRGB)?;
        let normals = load_image_rgba("Textures/Cerberus_N.tga", grr::Format::R8G8B8A8_UNORM)?;
        let metalness = load_image_rgba("Textures/Cerberus_M.tga", grr::Format::R8_UNORM)?;
        let roughness = load_image_rgba("Textures/Cerberus_R.tga", grr::Format::R8_UNORM)?;
        let occlusion = load_image_rgba("Textures/Raw/Cerberus_AO.tga", grr::Format::R8_UNORM)?;

        let sampler_trilinear = grr.create_sampler(grr::SamplerDesc {
            min_filter: grr::Filter::Linear,
            mag_filter: grr::Filter::Linear,
            mip_map: Some(grr::Filter::Linear),
            address: (
                grr::SamplerAddress::Repeat,
                grr::SamplerAddress::Repeat,
                grr::SamplerAddress::Repeat,
            ),
            lod_bias: 0.0,
            lod: 0.0..1024.0,
            compare: None,
            border_color: [0.0, 0.0, 0.0, 1.0],
        })?;

        let pbr_vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/pbr.vs"),
            grr::ShaderFlags::VERBOSE,
        )?;
        let pbr_fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/pbr.fs"),
            grr::ShaderFlags::VERBOSE,
        )?;

        let pbr_pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: pbr_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(pbr_fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        let pbr_vertex_array = grr.create_vertex_array(&[
            grr::VertexAttributeDesc {
                location: 0,
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                offset: 0,
            },
            grr::VertexAttributeDesc {
                location: 1,
                binding: 0,
                format: grr::VertexFormat::Xy32Float,
                offset: 12,
            },
            grr::VertexAttributeDesc {
                location: 2,
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                offset: 20,
            },
        ])?;

        let depth_stencil_state = grr::DepthStencil {
            depth_test: true,
            depth_write: true,
            depth_compare_op: grr::Compare::LessEqual,
            stencil_test: false,
            stencil_front: grr::StencilFace::KEEP,
            stencil_back: grr::StencilFace::KEEP,
        };

        let depth_stencil_none = grr::DepthStencil {
            depth_test: false,
            depth_write: false,
            depth_compare_op: grr::Compare::Always,
            stencil_test: false,
            stencil_front: grr::StencilFace::KEEP,
            stencil_back: grr::StencilFace::KEEP,
        };

        println!("Loading HDR image from disk");
        let hdr_image = image::hdr::HdrDecoder::new(io::BufReader::new(
            fs::File::open(format!("{}/Lobby-Center_2k.hdr", base_path)).unwrap(),
        ))
        .unwrap();
        let hdr_image_width = hdr_image.metadata().width;
        let hdr_image_height = hdr_image.metadata().height;
        let hdr_image_data = hdr_image.read_image_hdr().unwrap();
        let hdr_image_raw = hdr_image_data
            .iter()
            .flat_map(|c| {
                let chn = c.channels();
                vec![chn[0], chn[1], chn[2]]
            })
            .collect::<Vec<_>>();

        println!(
            "w: {}, h: {}, data_len: {}",
            hdr_image_width,
            hdr_image_height,
            hdr_image_raw.len()
        );

        let hdr_image_levels = max_mip_levels_2d(hdr_image_width, hdr_image_height);
        let hdr_texture = grr.create_image(
            grr::ImageType::D2 {
                width: hdr_image_width,
                height: hdr_image_height,
                layers: 1,
                samples: 1,
            },
            grr::Format::R16G16B16_SFLOAT,
            hdr_image_levels,
        )?;

        println!("Uploading HDR image into GPU memory");
        grr.copy_host_to_image(
            &hdr_image_raw,
            hdr_texture,
            grr::HostImageCopy {
                host_layout: grr::MemoryLayout {
                    base_format: grr::BaseFormat::RGB,
                    format_layout: grr::FormatLayout::F32,
                    row_length: hdr_image_width,
                    image_height: hdr_image_height,
                    alignment: 4,
                },
                image_subresource: grr::SubresourceLayers {
                    level: 0,
                    layers: 0..1,
                },
                image_offset: grr::Offset { x: 0, y: 0, z: 0 },
                image_extent: grr::Extent {
                    width: hdr_image_width,
                    height: hdr_image_height,
                    depth: 1,
                },
            },
        );

        grr.generate_mipmaps(hdr_texture);

        let hdr_sampler = grr.create_sampler(grr::SamplerDesc {
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

        let empty_vertex_array = grr.create_vertex_array(&[])?;

        println!("Creating Env Cubemap");
        let env_size = 512;
        let env_cubmap = grr.create_image(
            grr::ImageType::D2 {
                width: env_size,
                height: env_size,
                layers: 6,
                samples: 1,
            },
            grr::Format::R16G16B16_SFLOAT,
            1,
        )?;

        let env_cubemap_view = grr.create_image_view(
            env_cubmap,
            grr::ImageViewType::Cube,
            grr::Format::R16G16B16_SFLOAT,
            grr::SubresourceRange {
                layers: 0..6,
                levels: 0..1,
            },
        )?;

        let env_cubemap_sampler = grr.create_sampler(grr::SamplerDesc {
            min_filter: grr::Filter::Linear,
            mag_filter: grr::Filter::Linear,
            mip_map: Some(grr::Filter::Linear),
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

        let env_proj_fbo = grr.create_framebuffer()?;

        let env_proj = glm::perspective(1.0, glm::half_pi(), 0.1, 10.0);
        let env_eye = glm::vec3(0.0, 0.0, 0.0);
        let env_views = [
            glm::look_at(
                &env_eye,
                &glm::vec3(1.0, 0.0, 0.0),
                &glm::vec3(0.0, 1.0, 0.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(-1.0, 0.0, 0.0),
                &glm::vec3(0.0, 1.0, 0.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, -1.0, 0.0),
                &glm::vec3(0.0, 0.0, -1.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, 1.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, 0.0, -1.0),
                &glm::vec3(0.0, 1.0, 0.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, 0.0, 1.0),
                &glm::vec3(0.0, 1.0, 0.0),
            ),
        ];

        let env_views_inv = [
            glm::look_at(
                &env_eye,
                &glm::vec3(1.0, 0.0, 0.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(-1.0, 0.0, 0.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, 1.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, -1.0, 0.0),
                &glm::vec3(0.0, 0.0, -1.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, 0.0, 1.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
            glm::look_at(
                &env_eye,
                &glm::vec3(0.0, 0.0, -1.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
        ];

        // Project HDR env to cubmap
        let cubemap_proj_vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap.vs"),
            grr::ShaderFlags::VERBOSE,
        )?;
        let cubemap_proj_fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap_proj.fs"),
            grr::ShaderFlags::VERBOSE,
        )?;

        let cubemap_proj_pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: cubemap_proj_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(cubemap_proj_fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        grr.bind_draw_framebuffer(env_proj_fbo);
        grr.set_color_attachments(env_proj_fbo, &[0]);
        grr.set_viewport(
            0,
            &[grr::Viewport {
                x: 0.0,
                y: 0.0,
                w: env_size as _,
                h: env_size as _,
                n: 0.0,
                f: 1.0,
            }],
        );
        grr.set_scissor(
            0,
            &[grr::Region {
                x: 0,
                y: 0,
                w: env_size as _,
                h: env_size as _,
            }],
        );
        grr.bind_pipeline(cubemap_proj_pipeline);
        grr.bind_vertex_array(empty_vertex_array);
        grr.bind_image_views(0, &[hdr_texture.as_view()]);
        grr.bind_samplers(0, &[hdr_sampler]);

        for i in 0..6 {
            let env_cubmap_layer = grr.create_image_view(
                env_cubmap,
                grr::ImageViewType::D2,
                grr::Format::R16G16B16_SFLOAT,
                grr::SubresourceRange {
                    layers: i..i + 1,
                    levels: 0..1,
                },
            )?;

            grr.bind_attachments(
                env_proj_fbo,
                &[(
                    grr::Attachment::Color(0),
                    grr::AttachmentView::Image(env_cubmap_layer),
                )],
            );

            let face_view = &env_views[i as usize];
            grr.bind_uniform_constants(
                cubemap_proj_pipeline,
                0,
                &[
                    grr::Constant::Mat4x4(glm::inverse(&env_proj).into()),
                    grr::Constant::Mat3x3(glm::mat4_to_mat3(&glm::inverse(face_view)).into()),
                ],
            );

            grr.draw(grr::Primitive::Triangles, 0..3, 0..1);
        }

        // Pre-Pass: Split sum: BRDF integration
        let brdf_integration_vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/brdf_integration.vs"),
            grr::ShaderFlags::VERBOSE,
        )?;
        let brdf_integration_fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/brdf_integration.fs"),
            grr::ShaderFlags::VERBOSE,
        )?;

        let brdf_integration_pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: brdf_integration_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(brdf_integration_fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        let brdf_lut = grr.create_image(
            grr::ImageType::D2 {
                width: env_size,
                height: env_size,
                layers: 1,
                samples: 1,
            },
            grr::Format::R16G16_SFLOAT,
            1,
        )?;

        let brdf_lut_sampler = grr.create_sampler(grr::SamplerDesc {
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

        let brdf_fbo = grr.create_framebuffer()?;
        grr.bind_pipeline(brdf_integration_pipeline);
        grr.bind_draw_framebuffer(brdf_fbo);
        grr.bind_attachments(
            brdf_fbo,
            &[(
                grr::Attachment::Color(0),
                grr::AttachmentView::Image(brdf_lut.as_view()),
            )],
        );
        grr.set_color_attachments(brdf_fbo, &[0]);
        grr.set_viewport(
            0,
            &[grr::Viewport {
                x: 0.0,
                y: 0.0,
                w: env_size as _,
                h: env_size as _,
                n: 0.0,
                f: 1.0,
            }],
        );
        grr.set_scissor(
            0,
            &[grr::Region {
                x: 0,
                y: 0,
                w: env_size as _,
                h: env_size as _,
            }],
        );
        grr.draw(grr::Primitive::Triangles, 0..3, 0..1);

        context.swap_buffers();

        // Pre-Pass: Env map irradiance convolution
        let env_irradiance_vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap.vs"),
            grr::ShaderFlags::VERBOSE,
        )?;
        let env_irradiance_fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap_irradiance.fs"),
            grr::ShaderFlags::VERBOSE,
        )?;

        let env_irradiance_pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: env_irradiance_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(env_irradiance_fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        let env_irradiance_size = 32;
        let env_irradiance = grr.create_image(
            grr::ImageType::D2 {
                width: env_irradiance_size,
                height: env_irradiance_size,
                layers: 6,
                samples: 1,
            },
            grr::Format::R16G16B16_SFLOAT,
            1,
        )?;
        let env_irradiance_view = grr.create_image_view(
            env_irradiance,
            grr::ImageViewType::Cube,
            grr::Format::R16G16B16_SFLOAT,
            grr::SubresourceRange {
                layers: 0..6,
                levels: 0..1,
            },
        )?;
        let env_irradiance_sampler = grr.create_sampler(grr::SamplerDesc {
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

        let env_irradiance_fbo = grr.create_framebuffer()?;
        grr.bind_draw_framebuffer(env_irradiance_fbo);
        grr.set_color_attachments(env_irradiance_fbo, &[0]);
        grr.set_viewport(
            0,
            &[grr::Viewport {
                x: 0.0,
                y: 0.0,
                w: env_irradiance_size as _,
                h: env_irradiance_size as _,
                n: 0.0,
                f: 1.0,
            }],
        );
        grr.set_scissor(
            0,
            &[grr::Region {
                x: 0,
                y: 0,
                w: env_irradiance_size as _,
                h: env_irradiance_size as _,
            }],
        );
        grr.bind_pipeline(env_irradiance_pipeline);
        grr.bind_image_views(0, &[env_cubemap_view]);
        grr.bind_samplers(0, &[env_cubemap_sampler]);

        for i in 0..6 {
            let env_irradiance_layer = grr.create_image_view(
                env_irradiance,
                grr::ImageViewType::D2,
                grr::Format::R16G16B16_SFLOAT,
                grr::SubresourceRange {
                    layers: i..i + 1,
                    levels: 0..1,
                },
            )?;

            grr.bind_attachments(
                env_irradiance_fbo,
                &[(
                    grr::Attachment::Color(0),
                    grr::AttachmentView::Image(env_irradiance_layer),
                )],
            );

            let face_view = &env_views_inv[i as usize];
            grr.bind_uniform_constants(
                env_irradiance_pipeline,
                0,
                &[
                    grr::Constant::Mat4x4(glm::inverse(&env_proj).into()),
                    grr::Constant::Mat3x3(glm::mat4_to_mat3(&glm::inverse(face_view)).into()),
                ],
            );

            grr.draw(grr::Primitive::Triangles, 0..3, 0..1);
        }

        // Pre-Pass: Prefiltered specular env map
        let env_prefilter_vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap.vs"),
            grr::ShaderFlags::VERBOSE,
        )?;
        let env_prefilter_fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap_specular_filtered.fs"),
            grr::ShaderFlags::VERBOSE,
        )?;

        let env_prefilter_pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: env_prefilter_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(env_prefilter_fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        let num_prefiltered_levels = 5;
        let env_prefiltered_size = 128;
        let env_prefiltered = grr.create_image(
            grr::ImageType::D2 {
                width: env_prefiltered_size,
                height: env_prefiltered_size,
                layers: 6,
                samples: 1,
            },
            grr::Format::R16G16B16_SFLOAT,
            num_prefiltered_levels,
        )?;
        let env_prefiltered_view = grr.create_image_view(
            env_prefiltered,
            grr::ImageViewType::Cube,
            grr::Format::R16G16B16_SFLOAT,
            grr::SubresourceRange {
                layers: 0..6,
                levels: 0..num_prefiltered_levels,
            },
        )?;

        let env_prefiltered_sampler = grr.create_sampler(grr::SamplerDesc {
            min_filter: grr::Filter::Linear,
            mag_filter: grr::Filter::Linear,
            mip_map: Some(grr::Filter::Linear),
            address: (
                grr::SamplerAddress::ClampEdge,
                grr::SamplerAddress::ClampEdge,
                grr::SamplerAddress::ClampEdge,
            ),
            lod_bias: 0.0,
            lod: 0.0..1024.0,
            compare: None,
            border_color: [0.0, 0.0, 0.0, 1.0],
        })?;

        let env_prefilter_fbo = grr.create_framebuffer()?;
        grr.bind_pipeline(env_prefilter_pipeline);
        grr.bind_image_views(0, &[env_cubemap_view]);
        grr.bind_samplers(0, &[env_cubemap_sampler]);

        grr.bind_draw_framebuffer(env_prefilter_fbo);
        grr.set_color_attachments(env_prefilter_fbo, &[0]);

        let mut level_dim = env_prefiltered_size;

        grr.bind_uniform_constants(
            env_prefilter_pipeline,
            0,
            &[grr::Constant::Mat4x4(glm::inverse(&env_proj).into())],
        );

        for mip in 0..num_prefiltered_levels {
            grr.set_viewport(
                0,
                &[grr::Viewport {
                    x: 0.0,
                    y: 0.0,
                    w: level_dim as _,
                    h: level_dim as _,
                    n: 0.0,
                    f: 1.0,
                }],
            );
            grr.set_scissor(
                0,
                &[grr::Region {
                    x: 0,
                    y: 0,
                    w: level_dim as _,
                    h: level_dim as _,
                }],
            );

            let roughness = mip as f32 / (num_prefiltered_levels - 1) as f32;
            grr.bind_uniform_constants(env_prefilter_pipeline, 2, &[grr::Constant::F32(roughness)]);

            for face in 0..6 {
                let face_view = &env_views_inv[face as usize];

                grr.bind_uniform_constants(
                    env_prefilter_pipeline,
                    1,
                    &[grr::Constant::Mat3x3(
                        glm::mat4_to_mat3(&glm::inverse(face_view)).into(),
                    )],
                );

                let env_cubmap_slice = grr.create_image_view(
                    env_prefiltered,
                    grr::ImageViewType::D2,
                    grr::Format::R16G16B16_SFLOAT,
                    grr::SubresourceRange {
                        layers: face..face + 1,
                        levels: mip..mip + 1,
                    },
                )?;

                grr.bind_attachments(
                    env_prefilter_fbo,
                    &[(
                        grr::Attachment::Color(0),
                        grr::AttachmentView::Image(env_cubmap_slice),
                    )],
                );

                grr.draw(grr::Primitive::Triangles, 0..3, 0..1);
            }

            level_dim /= 2;
        }

        // Pass: skybox background
        let skybox_vs = grr.create_shader(
            grr::ShaderStage::Vertex,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/cubemap.vs"),
            grr::ShaderFlags::VERBOSE,
        )?;
        let skybox_fs = grr.create_shader(
            grr::ShaderStage::Fragment,
            grr::ShaderSource::Glsl,
            include_bytes!("assets/Shaders/skybox.fs"),
            grr::ShaderFlags::VERBOSE,
        )?;

        let skybox_pipeline = grr.create_graphics_pipeline(
            grr::VertexPipelineDesc {
                vertex_shader: skybox_vs,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: Some(skybox_fs),
            },
            grr::PipelineFlags::VERBOSE,
        )?;

        // Scene description
        let mut camera =
            camera::Camera::new(glm::vec3(-16.0, 12.0, -50.0), glm::vec3(-0.1, 3.3, 0.0));

        let mut frame_time = FrameTime::new();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::LoopDestroyed => {
                    grr.delete_images(&[albedo, normals, occlusion, metalness, roughness]);
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    camera.handle_event(input);
                }
                Event::MainEventsCleared => {
                    let size = window.inner_size();
                    let dt = frame_time.update();
                    camera.update(dt);

                    let perspective = glm::perspective(
                        size.width as f32 / size.height as f32,
                        glm::half_pi::<f32>() * 0.8,
                        0.1,
                        1024.0,
                    );
                    let view = camera.view();
                    let model = glm::rotation(-glm::half_pi::<f32>(), &glm::vec3(1.0, 0.0, 0.0));

                    grr.bind_draw_framebuffer(grr::Framebuffer::DEFAULT);
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

                    grr.clear_attachment(
                        grr::Framebuffer::DEFAULT,
                        grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]),
                    );
                    grr.clear_attachment(
                        grr::Framebuffer::DEFAULT,
                        grr::ClearAttachment::Depth(1.0),
                    );

                    // Skybox pass
                    grr.bind_pipeline(skybox_pipeline);
                    grr.bind_depth_stencil_state(&depth_stencil_none);
                    grr.bind_image_views(0, &[env_cubemap_view]);
                    grr.bind_samplers(0, &[env_cubemap_sampler]);
                    grr.bind_uniform_constants(
                        skybox_pipeline,
                        0,
                        &[
                            grr::Constant::Mat4x4(glm::inverse(&perspective).into()),
                            grr::Constant::Mat3x3(glm::mat4_to_mat3(&glm::inverse(&view)).into()),
                        ],
                    );
                    grr.draw(grr::Primitive::Triangles, 0..3, 0..1);

                    grr.bind_pipeline(pbr_pipeline);
                    grr.bind_vertex_array(pbr_vertex_array);
                    grr.bind_vertex_buffers(
                        pbr_vertex_array,
                        0,
                        &[grr::VertexBufferView {
                            buffer: mesh_data,
                            offset: 0,
                            stride: mem::size_of::<Vertex>() as _,
                            input_rate: grr::InputRate::Vertex,
                        }],
                    );
                    grr.bind_index_buffer(pbr_vertex_array, index_data);
                    grr.bind_uniform_constants(
                        pbr_pipeline,
                        0,
                        &[
                            grr::Constant::Mat4x4(perspective.into()),
                            grr::Constant::Mat4x4(view.into()),
                            grr::Constant::Mat4x4(model.into()),
                            grr::Constant::Vec3(camera.position().into()),
                        ],
                    );
                    grr.bind_image_views(
                        0,
                        &[
                            albedo.as_view(),
                            normals.as_view(),
                            metalness.as_view(),
                            roughness.as_view(),
                            occlusion.as_view(),
                            brdf_lut.as_view(),
                            env_prefiltered_view,
                            env_irradiance_view,
                        ],
                    );
                    grr.bind_samplers(
                        0,
                        &[
                            sampler_trilinear,
                            sampler_trilinear,
                            sampler_trilinear,
                            sampler_trilinear,
                            sampler_trilinear,
                            brdf_lut_sampler,
                            env_prefiltered_sampler,
                            env_irradiance_sampler,
                        ],
                    );
                    grr.bind_depth_stencil_state(&depth_stencil_state);

                    for geometry in &geometries {
                        grr.draw_indexed(
                            grr::Primitive::Triangles,
                            grr::IndexTy::U32,
                            geometry.base_index..geometry.base_index + geometry.num_indices,
                            0..1,
                            geometry.base_vertex,
                        );
                    }

                    context.swap_buffers();
                }
                _ => (),
            }
        })
    }
}
