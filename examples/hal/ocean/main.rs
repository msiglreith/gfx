#![cfg_attr(
    not(any(feature = "vulkan", feature = "dx12")),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

extern crate env_logger;
extern crate cgmath;
extern crate gfx_hal as hal;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate winit;
extern crate time;
extern crate panopaea;

use hal::{buffer, command, device as d, image as i, memory as m, pass, pool, pso, state};
use hal::{
    Adapter, Device, FrameSync, Gpu, Instance, QueueType, Submission, Surface,
    DescriptorPool, IndexType, Primitive, Swapchain, SwapchainConfig, Backbuffer};
use hal::command::{ClearColor, ClearValue};
use hal::format::{self, Format, Formatted, Srgba8 as ColorFormat, Swizzle, Vec2, Vec3};
use hal::target::Rect;

use panopaea::ocean::empirical;

#[cfg(any(feature = "vulkan", feature = "dx12"))]
use ocean::{CorrectionLocals, PropagateLocals};

mod camera;
#[cfg(any(feature = "vulkan", feature = "dx12"))]
mod fft;
#[cfg(any(feature = "vulkan", feature = "dx12"))]
mod ocean;

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Vertex {
    a_Pos: [f32; 3],
    a_Uv: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
struct PatchOffset {
    a_offset: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
struct Locals {
    a_proj: [[f32; 4]; 4],
    a_view: [[f32; 4]; 4],
}

const RESOLUTION: usize = 512;
const HALF_RESOLUTION: usize = 128;

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: i::ASPECT_COLOR,
    levels: 0 .. 1,
    layers: 0 .. 1,
};

#[cfg(any(feature = "vulkan", feature = "dx12"))]
fn main() {
    env_logger::init().unwrap();
    let mut events_loop = winit::EventsLoop::new();
    let wb = winit::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title("ocean".to_string());
    let window = wb
        .build(&events_loop)
        .unwrap();

    let window_size = window.get_inner_size_pixels().unwrap();
    let pixel_width = window_size.0 as u16;
    let pixel_height = window_size.1 as u16;

    // camera
    let view_pos = cgmath::Point3::new(-110.0, 150.0, 200.0);
    let view_angles = (cgmath::Rad(-1.28), cgmath::Rad(-0.44), cgmath::Rad(0.0));
    let mut camera = camera::Camera::new(view_pos, view_angles, cgmath::Vector3::new(0.0, 1.0, 0.0));

    let perspective: [[f32; 4]; 4] = {
        let aspect_ratio = pixel_width as f32 / pixel_height as f32;
        cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 1.0, 1024.0).into()
    };

    let (_instance, adapters, mut surface) = {
        let instance = back::Instance::create("gfx-rs ocean", 1);
        let surface = instance.create_surface(&window);
        let adapters = instance.enumerate_adapters();
        (instance, adapters, surface)
    };

    let adapter = &adapters[0];
    let Gpu { mut device, mut general_queues, memory_types, .. } =
        adapter.open_with(|ref family, qtype| {
            if qtype.supports_compute() && qtype.supports_graphics() && surface.supports_queue(family) {
                (1, QueueType::General)
            } else {
                (0, QueueType::Transfer)
            }
        });
    let mut queue = general_queues.remove(0);

    let swap_config = SwapchainConfig::new()
        .with_color::<ColorFormat>();
    let (mut swap_chain, backbuffer) = surface.build_swapchain(swap_config, &queue);

    let frame_images = match backbuffer {
        Backbuffer::Images(images) => {
            images
                .into_iter()
                .map(|image| {
                    let rtv = device.create_image_view(&image, ColorFormat::SELF, Swizzle::NO, COLOR_RANGE).unwrap();
                    (image, rtv)
                })
                .collect::<Vec<_>>()
        }
        _ => unimplemented!(),
    };

    let mut frame_semaphore = device.create_semaphore();
    let mut frame_fence = device.create_fence(false);

    #[cfg(feature = "vulkan")]
    let vs_ocean = device
        .create_shader_module_from_glsl(
            include_str!("shader/ocean.vert"),
            pso::Stage::Vertex,
        ).unwrap();
    #[cfg(feature = "vulkan")]
    let fs_ocean = device
        .create_shader_module_from_glsl(
            include_str!("shader/ocean.frag"),
            pso::Stage::Fragment,
        ).unwrap();

    #[cfg(feature = "dx12")]
    let vs_ocean = device
        .create_shader_module_from_source(
            pso::Stage::Vertex,
            "ocean_vs", // TODO
            "main",
            include_bytes!("shader/ocean.hlsl"),
        ).unwrap();
    #[cfg(feature = "dx12")]
    let fs_ocean = device
        .create_shader_module_from_source(
            pso::Stage::Fragment,
            "ocean_ps", // TODO
            "main",
            include_bytes!("shader/ocean.hlsl"),
        ).unwrap();

    let fft = fft::Fft::init(&mut device);
    let propagate = ocean::Propagation::init(&mut device);
    let correction = ocean::Correction::init(&mut device);

    let set_layout = device.create_descriptor_set_layout(&[
            pso::DescriptorSetLayoutBinding {
                binding: 0,
                ty: pso::DescriptorType::UniformBuffer,
                count: 1,
                stage_flags: pso::STAGE_VERTEX,
            },
            pso::DescriptorSetLayoutBinding {
                binding: 1,
                ty: pso::DescriptorType::SampledImage,
                count: 1,
                stage_flags: pso::STAGE_VERTEX,
            },
            pso::DescriptorSetLayoutBinding {
                binding: 2,
                ty: pso::DescriptorType::Sampler,
                count: 1,
                stage_flags: pso::STAGE_VERTEX,
            },
        ],
    );

    let ocean_layout = device.create_pipeline_layout(&[&set_layout]);
    let ocean_pass = {
        let attachment = pass::Attachment {
            format: ColorFormat::SELF,
            ops: pass::AttachmentOps::new(pass::AttachmentLoadOp::Clear, pass::AttachmentStoreOp::Store),
            stencil_ops: pass::AttachmentOps::DONT_CARE,
            layouts: i::ImageLayout::Undefined .. i::ImageLayout::Present,
        };

        let subpass = pass::SubpassDesc {
            colors: &[(0, i::ImageLayout::ColorAttachmentOptimal)],
            depth_stencil: None,
            inputs: &[],
            preserves: &[],
        };

        device.create_renderpass(&[attachment], &[subpass], &[])
    };

    let extent = d::Extent { width: pixel_width as _, height: pixel_height as _, depth: 1 };
    let ocean_framebuffers = frame_images
        .iter()
        .map(|&(_, ref rtv)| {
            device.create_framebuffer(&ocean_pass, &[rtv], extent).unwrap()
        })
        .collect::<Vec<_>>();

    let mut ocean_pipe_desc = pso::GraphicsPipelineDesc::new(
        Primitive::TriangleList,
        pso::Rasterizer {
            polgyon_mode: state::RasterMethod::Line(1),
            cull_mode: state::CullFace::Nothing,
            front_face: state::FrontFace::CounterClockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
        }
    );
    ocean_pipe_desc.blender.targets.push(pso::ColorInfo {
        mask: state::MASK_ALL,
        color: None,
        alpha: None,
    });
    ocean_pipe_desc.vertex_buffers.push(pso::VertexBufferDesc {
        stride: std::mem::size_of::<Vertex>() as u32,
        rate: 0,
    });
    ocean_pipe_desc.vertex_buffers.push(pso::VertexBufferDesc {
        stride: std::mem::size_of::<PatchOffset>() as u32,
        rate: 1,
    });
    ocean_pipe_desc.attributes.push(pso::AttributeDesc {
        location: 0,
        binding: 0,
        element: pso::Element {
            format: <Vec3<f32> as Formatted>::SELF,
            offset: 0,
        },
    });
    ocean_pipe_desc.attributes.push(pso::AttributeDesc {
        location: 1,
        binding: 0,
        element: pso::Element {
            format: <Vec2<f32> as Formatted>::SELF,
            offset: 12,
        },
    });
    ocean_pipe_desc.attributes.push(pso::AttributeDesc {
        location: 2,
        binding: 1,
        element: pso::Element {
            format: <Vec2<f32> as Formatted>::SELF,
            offset: 0,
        },
    });

    let sampler = device.create_sampler(
        i::SamplerInfo::new(
            i::FilterMethod::Bilinear,
            i::WrapMode::Tile,
        )
    );

    let pipelines = {
        let (vs_entry, fs_entry) = (
            pso::EntryPoint { entry: "main", module: &vs_ocean },
            pso::EntryPoint { entry: "main", module: &fs_ocean },
        );

        let shader_entries = pso::GraphicsShaderSet {
            vertex: vs_entry,
            hull: None,
            domain: None,
            geometry: None,
            fragment: Some(fs_entry),
        };
        let subpass = pass::Subpass { index: 0, main_pass: &ocean_pass };
        device.create_graphics_pipelines(&[
            (shader_entries, &ocean_layout, subpass, &ocean_pipe_desc)
        ])
    };

    let mut desc_pool = device.create_descriptor_pool(
        1, // sets
        &[
            pso::DescriptorRangeDesc {
                ty: pso::DescriptorType::UniformBuffer,
                count: 1,
            },
            pso::DescriptorRangeDesc {
                ty: pso::DescriptorType::SampledImage,
                count: 1,
            },
            pso::DescriptorRangeDesc {
                ty: pso::DescriptorType::Sampler,
                count: 1,
            },
        ],
    );

    let desc_sets = desc_pool.allocate_sets(&[&set_layout]);

    let (locals_buffer, locals_memory) = {
        let buffer_stride = std::mem::size_of::<Locals>() as u64;
        let buffer_len = buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::UNIFORM).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let locals_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut locals = device
                .acquire_mapping_writer::<Locals>(&locals_buffer, 0..buffer_len)
                .unwrap();
            locals[0] = Locals {
                a_proj: perspective.into(),
                a_view: camera.view().into(),
            };
            device.release_mapping_writer(locals);
        }

        (locals_buffer, buffer_memory)
    };

    // grid
    let (grid_vertex_buffer, grid_vertex_memory) = {
        let buffer_stride = std::mem::size_of::<Vertex>() as u64;
        let buffer_len = (HALF_RESOLUTION * HALF_RESOLUTION) as u64 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::VERTEX).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let vertex_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut vertices = device
                .acquire_mapping_writer::<Vertex>(&vertex_buffer, 0..buffer_len)
                .unwrap();
            for z in 0..HALF_RESOLUTION {
                for x in 0..HALF_RESOLUTION {
                    vertices[z*HALF_RESOLUTION+x] = Vertex {
                        a_Pos: [x as f32, 0.0f32, z as f32],
                        a_Uv: [(x as f32) / (HALF_RESOLUTION-1) as f32, (z as f32) / (HALF_RESOLUTION-1) as f32],
                    };
                }
            }
            device.release_mapping_writer(vertices);
        }

        (vertex_buffer, buffer_memory)
    };

    let (grid_patch_buffer, grid_patch_memory) = {
        let buffer_stride = std::mem::size_of::<PatchOffset>() as u64;
        let buffer_len = 4 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::VERTEX).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut patch = device
                .acquire_mapping_writer::<PatchOffset>(&buffer, 0..buffer_len)
                .unwrap();
            patch[0] = PatchOffset {
                a_offset: [0.0, 0.0],
            };
            patch[1] = PatchOffset {
                a_offset: [(HALF_RESOLUTION-1) as f32, 0.0],
            };
            patch[2] = PatchOffset {
                a_offset: [0.0, (HALF_RESOLUTION-1) as f32],
            };
            patch[3] = PatchOffset {
                a_offset: [(HALF_RESOLUTION-1) as f32, (HALF_RESOLUTION-1) as f32],
            };
            device.release_mapping_writer(patch);
        }

        (buffer, buffer_memory)
    };

    let (grid_index_buffer, grid_index_memory) = {
        let buffer_stride = std::mem::size_of::<u32>() as u64;
        let buffer_len = (6 * (HALF_RESOLUTION-1) * (HALF_RESOLUTION-1)) as u64 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::INDEX).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let index_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut indices = device
                .acquire_mapping_writer::<u32>(&index_buffer, 0..buffer_len)
                .unwrap();
            for z in 0..HALF_RESOLUTION-1 {
                for x in 0..HALF_RESOLUTION-1 {
                    let i = z*(HALF_RESOLUTION-1)+x;
                    indices[6*i  ] = (z * HALF_RESOLUTION + x) as _;
                    indices[6*i+1] = ((z+1) * HALF_RESOLUTION + x) as _;
                    indices[6*i+2] = (z * HALF_RESOLUTION + x + 1) as _;
                    indices[6*i+3] = (z * HALF_RESOLUTION + x + 1) as _;
                    indices[6*i+4] = ((z+1) * HALF_RESOLUTION + x) as _;
                    indices[6*i+5] = ((z+1) * HALF_RESOLUTION + x + 1) as _;
                }
            }
            device.release_mapping_writer(indices);
        }

        (index_buffer, buffer_memory)
    };

    // ocean data
    let (initial_spec, dx_spec, dy_spec, dz_spec, spectrum_memory) = {
        let buffer_stride = 2 * std::mem::size_of::<f32>() as u64;
        let buffer_len = (RESOLUTION * RESOLUTION) as u64 * buffer_stride;

        let initial_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::STORAGE|buffer::TRANSFER_DST).unwrap();
        let dy_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::STORAGE).unwrap();
        let dx_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::STORAGE).unwrap();
        let dz_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::STORAGE).unwrap();

        let buffer_req = device.get_buffer_requirements(&initial_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::DEVICE_LOCAL)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, 16 * buffer_req.size).unwrap();
        let initial_spec = device.bind_buffer_memory(&buffer_memory, 0, initial_unbound).unwrap();

        let dx_offset = align_up(buffer_len, buffer_req.alignment);
        let dy_offset = align_up(dx_offset + buffer_len, buffer_req.alignment);
        let dz_offset = align_up(dy_offset + buffer_len, buffer_req.alignment);
        let dx_spec = device.bind_buffer_memory(&buffer_memory, dx_offset, dx_unbound).unwrap();
        let dy_spec = device.bind_buffer_memory(&buffer_memory, dy_offset, dy_unbound).unwrap();
        let dz_spec = device.bind_buffer_memory(&buffer_memory, dz_offset, dz_unbound).unwrap();

        (initial_spec, dx_spec, dy_spec, dz_spec, buffer_memory)
    };

    let (omega_buffer, omega_memory) = {
        let buffer_stride = std::mem::size_of::<f32>() as u64;
        let buffer_len = (RESOLUTION * RESOLUTION) as u64 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::STORAGE|buffer::TRANSFER_DST).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::DEVICE_LOCAL)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let omega_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        (omega_buffer, buffer_memory)
    };

    let (propagate_locals_buffer, propagate_locals_memory) = {
        let buffer_stride = std::mem::size_of::<PropagateLocals>() as u64;
        let buffer_len = buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::UNIFORM).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let locals_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();
        (locals_buffer, buffer_memory)
    };

    let (correct_locals_buffer, correct_locals_memory) = {
        let buffer_stride = std::mem::size_of::<CorrectionLocals>() as u64;
        let buffer_len = buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::UNIFORM).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let locals_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut locals = device
                .acquire_mapping_writer::<CorrectionLocals>(&locals_buffer, 0..buffer_len)
                .unwrap();
            locals[0] = CorrectionLocals {
                resolution: RESOLUTION as _,
            };
            device.release_mapping_writer(locals);
        }

        (locals_buffer, buffer_memory)
    };

    let viewport = hal::Viewport {
        x: 0, y: 0,
        w: pixel_width, h: pixel_height,
        near: 0.0, far: 1.0,
    };
    let scissor = Rect {
        x: 0, y: 0,
        w: pixel_width, h: pixel_height,
    };

    let spectrum_len = RESOLUTION*RESOLUTION*2*std::mem::size_of::<f32>();
    let omega_len = RESOLUTION*RESOLUTION*std::mem::size_of::<f32>();
    let mut general_pool = queue.create_general_pool(4, pool::CommandPoolCreateFlags::empty());

    // Initialize ocean..
    let parameters = empirical::Parameters {
        water_depth: 100.0,
        fetch: 800.0 * 1000.0,
        wind_speed: 25.0,

        water_density: 1000.0,
        surface_tension: 0.072,
        gravity: 9.81,

        swell: 0.25,
        domain_size: 1000.0,
    };

    let spectrum = empirical::SpectrumTMA {
        jonswap: empirical::SpectrumJONSWAP {
            wind_speed: parameters.wind_speed,
            fetch: parameters.fetch,
            gravity: parameters.gravity,
        },
        depth: parameters.water_depth,
    };

    let ocean = empirical::Ocean::<f32>::new(RESOLUTION);
    let (height_spectrum, omega) = empirical::build_height_spectrum(&parameters, &spectrum, RESOLUTION);

    // Upload initial data
    let (omega_staging_buffer, omega_staging_memory) = {
        let buffer_stride = std::mem::size_of::<f32>() as u64;
        let buffer_len = (RESOLUTION * RESOLUTION) as u64 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::TRANSFER_SRC).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let staging_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut data = device
                .acquire_mapping_writer::<f32>(&staging_buffer, 0..buffer_len)
                .unwrap();
            data.copy_from_slice(&omega.into_raw_vec());
            device.release_mapping_writer(data);
        }

        (staging_buffer, buffer_memory)
    };

    let (spec_staging_buffer, spec_staging_memory) = {
        let buffer_stride = 2*std::mem::size_of::<f32>() as u64;
        let buffer_len = (RESOLUTION * RESOLUTION) as u64 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer_stride, buffer::TRANSFER_SRC).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let mem_type =
            memory_types.iter().find(|mem_type| {
                buffer_req.type_mask & (1 << mem_type.id) != 0 &&
                mem_type.properties.contains(m::CPU_VISIBLE)
            })
            .unwrap();

        let buffer_memory = device.allocate_memory(mem_type, buffer_req.size).unwrap();
        let staging_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut data = device
                .acquire_mapping_writer::<[f32; 2]>(&staging_buffer, 0..buffer_len)
                .unwrap();
            // TODO: slow
            let spectrum = height_spectrum
                .into_raw_vec()
                .iter()
                .map(|c| [c.re, c.im])
                .collect::<Vec<_>>();

            data.copy_from_slice(&spectrum);
            device.release_mapping_writer(data);
        }

        (staging_buffer, buffer_memory)
    };

    let kind = i::Kind::D2(RESOLUTION as i::Size, RESOLUTION as i::Size, i::AaMode::Single);
    let img_format = Format(format::SurfaceType::R32_G32_B32_A32, format::ChannelType::Float);
    let image_unbound = device.create_image(kind, 1, img_format, i::SAMPLED|i::STORAGE).unwrap(); // TODO: usage
    let image_req = device.get_image_requirements(&image_unbound);

    let device_type = memory_types
        .iter()
        .find(|memory_type| {
            image_req.type_mask & (1 << memory_type.id) != 0 &&
            memory_type.properties.contains(m::DEVICE_LOCAL)
        })
        .unwrap();
    let image_memory = device.allocate_memory(device_type, image_req.size).unwrap();
    let displacement_map = device.bind_image_memory(&image_memory, 0, image_unbound).unwrap();
    let displacement_uav = device.create_image_view(&displacement_map, img_format, Swizzle::NO, COLOR_RANGE).unwrap();
    let displacement_srv = device.create_image_view(&displacement_map, img_format, Swizzle::NO, COLOR_RANGE).unwrap();

    // Upload data
    {
        let submit = {
            let mut cmd_buffer = general_pool.acquire_command_buffer();

            let image_barrier = m::Barrier::Image {
                states: (i::Access::empty(), i::ImageLayout::Undefined) ..
                        (i::SHADER_READ|i::SHADER_WRITE, i::ImageLayout::General),
                target: &displacement_map,
                range: COLOR_RANGE,
            };
            cmd_buffer.pipeline_barrier(pso::TOP_OF_PIPE .. pso::COMPUTE_SHADER, &[image_barrier]);

            // TODO: pipeline barriers
            cmd_buffer.copy_buffer(
                &spec_staging_buffer,
                &initial_spec,
                &[command::BufferCopy {
                    src: 0,
                    dst: 0,
                    size: (RESOLUTION * RESOLUTION * 2*std::mem::size_of::<f32>()) as u64,
                }]);

            cmd_buffer.copy_buffer(
                &omega_staging_buffer,
                &omega_buffer,
                &[command::BufferCopy {
                    src: 0,
                    dst: 0,
                    size: (RESOLUTION * RESOLUTION * std::mem::size_of::<f32>()) as u64,
                }]);
            cmd_buffer.finish()
        };

        let submission = Submission::new()
            .submit(&[submit]);
        queue.submit(submission, Some(&mut frame_fence));

        device.wait_for_fences(&[&frame_fence], d::WaitFor::All, !0);
    }

    device.update_descriptor_sets(&[
        pso::DescriptorSetWrite {
            set: &desc_sets[0],
            binding: 0,
            array_offset: 0,
            write: pso::DescriptorWrite::UniformBuffer(vec![
                (&locals_buffer, 0..std::mem::size_of::<Locals>() as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &desc_sets[0],
            binding: 1,
            array_offset: 0,
            write: pso::DescriptorWrite::SampledImage(vec![(&displacement_srv, i::ImageLayout::General)]),
        },
        pso::DescriptorSetWrite {
            set: &desc_sets[0],
            binding: 2,
            array_offset: 0,
            write: pso::DescriptorWrite::Sampler(vec![&sampler]),
        },

        pso::DescriptorSetWrite {
            set: &propagate.desc_sets[0],
            binding: 0,
            array_offset: 0,
            write: pso::DescriptorWrite::UniformBuffer(vec![
                (&propagate_locals_buffer, 0..std::mem::size_of::<PropagateLocals>() as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &propagate.desc_sets[0],
            binding: 1,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&initial_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &propagate.desc_sets[0],
            binding: 2,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&omega_buffer, 0..omega_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &propagate.desc_sets[0],
            binding: 3,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dy_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &propagate.desc_sets[0],
            binding: 4,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dx_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &propagate.desc_sets[0],
            binding: 5,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dz_spec, 0..spectrum_len as u64),
            ]),
        },
    ]);

    device.update_descriptor_sets(&[
        pso::DescriptorSetWrite {
            set: &correction.desc_sets[0],
            binding: 0,
            array_offset: 0,
            write: pso::DescriptorWrite::UniformBuffer(vec![
                (&correct_locals_buffer, 0..std::mem::size_of::<CorrectionLocals>() as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &correction.desc_sets[0],
            binding: 1,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dy_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &correction.desc_sets[0],
            binding: 2,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dx_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &correction.desc_sets[0],
            binding: 3,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dz_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &correction.desc_sets[0],
            binding: 4,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageImage(vec![
                (&displacement_uav, i::ImageLayout::General),
            ]),
        },
    ]);

    device.update_descriptor_sets(&[
        pso::DescriptorSetWrite {
            set: &fft.desc_sets[0],
            binding: 0,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dx_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &fft.desc_sets[1],
            binding: 0,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dy_spec, 0..spectrum_len as u64),
            ]),
        },
        pso::DescriptorSetWrite {
            set: &fft.desc_sets[2],
            binding: 0,
            array_offset: 0,
            write: pso::DescriptorWrite::StorageBuffer(vec![
                (&dz_spec, 0..spectrum_len as u64),
            ]),
        },
    ]);

    let time_start = time::PreciseTime::now();
    let mut time_last = time_start;

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event, .. } = event {
                match event {
                    winit::WindowEvent::KeyboardInput {
                        input: winit::KeyboardInput {
                            virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                            .. },
                        ..
                    } | winit::WindowEvent::Closed => running = false,
                    winit::WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        camera.on_event(input);
                    },
                    _ => (),
                }
            }
        });

        let time_now = time::PreciseTime::now();
        let time_elapsed_s = time_last.to(time_now).num_microseconds().unwrap() as f32 / 1_000_000.0;
        let time_current_s = time_start.to(time_now).num_microseconds().unwrap() as f32 / 1_000_000.0;
        time_last = time_now;

        device.reset_fences(&[&frame_fence]);
        general_pool.reset();
        let frame = swap_chain.acquire_frame(FrameSync::Semaphore(&mut frame_semaphore));

        // Update view
        camera.update(time_elapsed_s);
        let mut locals = device
            .acquire_mapping_writer::<Locals>(&locals_buffer, 0..std::mem::size_of::<Locals>() as u64)
            .unwrap();
        locals[0] = Locals {
            a_proj: perspective.into(),
            a_view: camera.view().into(),
        };
        device.release_mapping_writer(locals);

        let mut locals = device
            .acquire_mapping_writer::<PropagateLocals>(&propagate_locals_buffer, 0..std::mem::size_of::<PropagateLocals>() as u64)
            .unwrap();
        locals[0] = PropagateLocals {
            time: time_current_s,
            resolution: RESOLUTION as i32,
            domain_size: parameters.domain_size,
        };
        device.release_mapping_writer(locals);

        let submit = {
            let mut cmd_buffer = general_pool.acquire_command_buffer();

            cmd_buffer.bind_compute_pipeline(&propagate.pipeline);
            cmd_buffer.bind_compute_descriptor_sets(&propagate.layout, 0, &[&propagate.desc_sets[0]]);
            cmd_buffer.dispatch(RESOLUTION as u32, RESOLUTION as u32, 1);

            let dx_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dx_spec,
            };
            let dy_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dy_spec,
            };
            let dz_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dz_spec,
            };
            cmd_buffer.pipeline_barrier(
                pso::COMPUTE_SHADER .. pso::COMPUTE_SHADER,
                &[dx_barrier, dy_barrier, dz_barrier],
            );

            cmd_buffer.bind_compute_pipeline(&fft.row_pass);
            cmd_buffer.bind_compute_descriptor_sets(&fft.layout, 0, &[&fft.desc_sets[0]]);
            cmd_buffer.dispatch(1, RESOLUTION as u32, 1);
            cmd_buffer.bind_compute_descriptor_sets(&fft.layout, 0, &[&fft.desc_sets[1]]);
            cmd_buffer.dispatch(1, RESOLUTION as u32, 1);
            cmd_buffer.bind_compute_descriptor_sets(&fft.layout, 0, &[&fft.desc_sets[2]]);
            cmd_buffer.dispatch(1, RESOLUTION as u32, 1);

            let dx_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE|buffer::SHADER_READ..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dx_spec,
            };
            let dy_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE|buffer::SHADER_READ..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dy_spec,
            };
            let dz_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE|buffer::SHADER_READ..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dz_spec,
            };
            cmd_buffer.pipeline_barrier(
                pso::COMPUTE_SHADER .. pso::COMPUTE_SHADER,
                &[dx_barrier, dy_barrier, dz_barrier],
            );

            cmd_buffer.bind_compute_pipeline(&fft.col_pass);
            cmd_buffer.bind_compute_descriptor_sets(&fft.layout, 0, &[&fft.desc_sets[0]]);
            cmd_buffer.dispatch(1, RESOLUTION as u32, 1);
            cmd_buffer.bind_compute_descriptor_sets(&fft.layout, 0, &[&fft.desc_sets[1]]);
            cmd_buffer.dispatch(1, RESOLUTION as u32, 1);
            cmd_buffer.bind_compute_descriptor_sets(&fft.layout, 0, &[&fft.desc_sets[2]]);
            cmd_buffer.dispatch(1, RESOLUTION as u32, 1);

            let dx_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE|buffer::SHADER_READ..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dx_spec,
            };
            let dy_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE|buffer::SHADER_READ..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dy_spec,
            };
            let dz_barrier = m::Barrier::Buffer {
                states: buffer::SHADER_WRITE|buffer::SHADER_READ..buffer::SHADER_WRITE|buffer::SHADER_READ,
                target: &dz_spec,
            };
            let image_barrier = m::Barrier::Image {
                states: (i::SHADER_READ|i::SHADER_WRITE, i::ImageLayout::General) ..
                        (i::SHADER_READ|i::SHADER_WRITE, i::ImageLayout::General),
                target: &displacement_map,
                range: COLOR_RANGE,
            };
            cmd_buffer.pipeline_barrier(pso::VERTEX_SHADER|pso::COMPUTE_SHADER .. pso::COMPUTE_SHADER, &[dx_barrier, dy_barrier, dz_barrier, image_barrier]);

            cmd_buffer.bind_compute_pipeline(&correction.pipeline);
            cmd_buffer.bind_compute_descriptor_sets(&correction.layout, 0, &[&correction.desc_sets[0]]);
            cmd_buffer.dispatch(RESOLUTION as u32, RESOLUTION as u32, 1);

            let image_barrier = m::Barrier::Image {
                states: (i::SHADER_READ|i::SHADER_WRITE, i::ImageLayout::General) ..
                        (i::SHADER_READ|i::SHADER_WRITE, i::ImageLayout::General),
                target: &displacement_map,
                range: COLOR_RANGE,
            };
            cmd_buffer.pipeline_barrier(pso::COMPUTE_SHADER .. pso::VERTEX_SHADER, &[image_barrier]);

            cmd_buffer.set_viewports(&[viewport]);
            cmd_buffer.set_scissors(&[scissor]);
            cmd_buffer.bind_graphics_pipeline(&pipelines[0].as_ref().unwrap());
            cmd_buffer.bind_graphics_descriptor_sets(&ocean_layout, 0, &[&desc_sets[0]]);
            cmd_buffer.bind_vertex_buffers(pso::VertexBufferSet(vec![(&grid_vertex_buffer, 0), (&grid_patch_buffer, 0)]));
            cmd_buffer.bind_index_buffer(buffer::IndexBufferView {
                buffer: &grid_index_buffer,
                offset: 0,
                index_type: IndexType::U32,
            });

            {
                let mut encoder = cmd_buffer.begin_renderpass_inline(
                    &ocean_pass,
                    &ocean_framebuffers[frame.id()],
                    Rect { x: 0, y: 0, w: pixel_width, h: pixel_height },
                    &[ClearValue::Color(ClearColor::Float([0.6, 0.6, 0.6, 1.0]))],
                );
                let num_indices = 6*(HALF_RESOLUTION-1)*(HALF_RESOLUTION-1);
                encoder.draw_indexed(0..num_indices as u32, 0, 0..4);
            }

            cmd_buffer.finish()
        };

        let submission = Submission::new()
            .wait_on(&[(&mut frame_semaphore, pso::BOTTOM_OF_PIPE)])
            .submit(&[submit]);
        queue.submit(submission, Some(&mut frame_fence));

        device.wait_for_fences(&[&frame_fence], d::WaitFor::All, !0);
        swap_chain.present(&mut queue, &[]);
    }

    // cleanup
    fft.destroy(&mut device);
    propagate.destroy(&mut device);
    correction.destroy(&mut device);

    device.destroy_descriptor_pool(desc_pool);
    device.destroy_descriptor_set_layout(set_layout);

    device.destroy_shader_module(vs_ocean);
    device.destroy_shader_module(fs_ocean);

    device.destroy_buffer(grid_index_buffer);
    device.destroy_buffer(grid_vertex_buffer);
    device.destroy_buffer(locals_buffer);
    device.destroy_buffer(initial_spec);
    device.destroy_buffer(dx_spec);
    device.destroy_buffer(dy_spec);
    device.destroy_buffer(dz_spec);
    device.destroy_buffer(propagate_locals_buffer);

    device.free_memory(grid_index_memory);
    device.free_memory(grid_vertex_memory);
    device.free_memory(grid_patch_memory);
    device.free_memory(omega_memory);
    device.free_memory(locals_memory);
    device.free_memory(spectrum_memory);
    device.free_memory(propagate_locals_memory);
    device.free_memory(correct_locals_memory);
    device.free_memory(omega_staging_memory);
    device.free_memory(spec_staging_memory);

    for pipeline in pipelines {
        if let Ok(pipeline) = pipeline {
            device.destroy_graphics_pipeline(pipeline);
        }
    }

    for framebuffer in ocean_framebuffers {
        device.destroy_framebuffer(framebuffer);
    }

    for (image, rtv) in frame_images {
        device.destroy_image_view(rtv);
    }

    device.destroy_fence(frame_fence);
    device.destroy_semaphore(frame_semaphore);
}

fn align_up(value: u64, alignment: u64) -> u64 {
    ((value + alignment - 1) / alignment) * alignment
}

#[cfg(not(any(feature = "vulkan", feature = "dx12")))]
fn main() {
    println!("You need to enable the one of the following API backends: vulkan or dx12");
}
