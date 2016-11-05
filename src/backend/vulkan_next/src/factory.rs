// Copyright 2016 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core;
use core::mapping;
use core::memory;
use core::handle;
use core::buffer;
use core::factory;
use core::shade;
use core::state;
use core::pso;
use core::texture;
use core::handle::Producer;
use core::ShaderSet;

use std::slice;
use std::sync::Arc;
use std::ptr;
use std::mem;
use std::cell;
use vk;
use data;
use native;
use {Resources as R, Device, Share, Fence};

#[derive(Copy, Clone, Debug)]
pub struct MappingGate {
    pointer: *mut (),
}

impl mapping::Gate<R> for MappingGate {
    unsafe fn set<T>(&self, index: usize, val: T) {
        *(self.pointer as *mut T).offset(index as isize) = val;
    }

    unsafe fn slice<'a, 'b, T>(&'a self, len: usize) -> &'b [T] {
        slice::from_raw_parts(self.pointer as *const T, len)
    }

    unsafe fn mut_slice<'a, 'b, T>(&'a self, len: usize) -> &'b mut [T] {
        slice::from_raw_parts_mut(self.pointer as *mut T, len)
    }
}

pub struct Factory {
    device: Arc<Device>,
    share: Arc<Share>,
}

impl Factory {
    /// Create a new `Factory`.
    pub fn new(device: Arc<Device>, share: Arc<Share>) -> Factory {
        Factory {
            device: device,
            share: share,
        }
    }

    fn alloc(&self, usage: memory::Usage, reqs: vk::MemoryRequirements) -> vk::DeviceMemory {
        let info = vk::MemoryAllocateInfo {
            sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ptr::null(),
            allocationSize: reqs.size,
            memoryTypeIndex: 0, // TODO: high
        };
        let (dev, vk) = self.device.get();
        let mut mem = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.AllocateMemory(dev, &info, ptr::null(), &mut mem)
        });
        mem
    }

    pub fn view_image(&mut self, htex: &handle::RawTexture<R>, desc: texture::ResourceDesc, is_target: bool)
                    -> Result<native::ImageView, factory::ResourceViewError> {
        let raw_image = self.share.handles.borrow_mut().ref_texture(htex);
        let td = htex.get_info();
        let info = vk::ImageViewCreateInfo {
            sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            image: raw_image.image,
            viewType: match data::map_image_view_type(td.kind, desc.layer) {
                Ok(vt) => vt,
                Err(e) => return Err(factory::ResourceViewError::Layer(e)),
            },
            format: match data::map_format(td.format, desc.channel) {
                Some(f) => f,
                None => return Err(factory::ResourceViewError::Channel(desc.channel)),
            },
            components: data::map_swizzle(desc.swizzle),
            subresourceRange: vk::ImageSubresourceRange {
                aspectMask: data::map_image_aspect(td.format, desc.channel, is_target),
                baseMipLevel: desc.min as u32,
                levelCount: (desc.max + 1 - desc.min) as u32,
                baseArrayLayer: desc.layer.unwrap_or(0) as u32,
                layerCount: match desc.layer {
                    Some(_) => 1,
                    None => td.kind.get_num_slices().unwrap_or(1) as u32,
                },
            },
        };

        let (dev, vk) = self.device.get();
        let mut view = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateImageView(dev, &info, ptr::null(), &mut view)
        });
        Ok(native::ImageView {
            image: raw_image.image,
            view: view,
            layout: raw_image.layout.get(), //care!
            sub_range: info.subresourceRange,
        })
    }

    pub fn get_device(&self) -> &Arc<Device> {
        &self.device
    }

    fn get_shader_stages(&mut self, set: &ShaderSet<R>) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let mut stages = Vec::new();
        if let Some(vert) = set.get_vertex_entry() {
            stages.push(vk::PipelineShaderStageCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stage: vk::SHADER_STAGE_VERTEX_BIT,
                module: vert.get_shader(&mut self.share.handles.borrow_mut()).shader,
                pName: vert.get_entry_point().as_ptr() as *const i8,
                pSpecializationInfo: ptr::null(),
            });
        }
        if let Some(geom) = set.get_geometry_entry() {
            stages.push(vk::PipelineShaderStageCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stage: vk::SHADER_STAGE_GEOMETRY_BIT,
                module: geom.get_shader(&mut self.share.handles.borrow_mut()).shader,
                pName: geom.get_entry_point().as_ptr() as *const i8,
                pSpecializationInfo: ptr::null(),
            });
        }
        if let Some(pixel) = set.get_pixel_entry() {
            stages.push(vk::PipelineShaderStageCreateInfo {
                sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stage: vk::SHADER_STAGE_FRAGMENT_BIT,
                module: pixel.get_shader(&mut self.share.handles.borrow_mut()).shader,
                pName: pixel.get_entry_point().as_ptr() as *const i8,
                pSpecializationInfo: ptr::null(),
            });
        }
        stages
    }
}

impl factory::Factory<R> for Factory {
    fn allocate_memory(&mut self) {

    }

    fn create_renderpass(&mut self) -> handle::RenderPass<R> {
        unimplemented!()
    }

    fn create_pipeline_layout(&mut self) -> handle::PipelineLayout<R> {
        unimplemented!()
    }

    fn create_fence(&mut self, signaled: bool) -> handle::Fence<R> {
        let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: if signaled { vk::FENCE_CREATE_SIGNALED_BIT } else { 0 },
        };
        let (dev, vk) = self.device.get();
        let mut fence = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateFence(dev, &info, ptr::null(), &mut fence)
        });
        self.share.handles.borrow_mut().make_fence(Fence(fence))
    }

    fn create_shader(&mut self, code: &[u8]) -> Result<handle::Shader<R>, shade::CreateShaderError> {
        use core::handle::Producer;
        use mirror::reflect_spirv_module;
        use native::Shader;
        let info = vk::ShaderModuleCreateInfo {
            sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            codeSize: code.len(),
            pCode: code.as_ptr() as *const _,
        };
        let (dev, vk) = self.device.get();
        let mut shader = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateShaderModule(dev, &info, ptr::null(), &mut shader)
        });
        let reflection = reflect_spirv_module(code);
        let shader = Shader {
            shader: shader,
            reflection: reflection,
        };
        Ok(self.share.handles.borrow_mut().make_shader(shader))
    }

    fn create_compute_pipelines(&mut self) -> Vec<Result<handle::RawPipelineState<R>, pso::CreationError>> {
        unimplemented!()
    }

    fn create_graphics_pipelines(&mut self, infos: &[(&ShaderSet<R>, &pso::PipelineDesc)]) -> Vec<Result<handle::RawPipelineState<R>, pso::CreationError>> {
        let create_infos = infos.iter().map(|&(shader_set, desc)| {
            let stages = self.get_shader_stages(shader_set);
            let (polygon, line_width) = data::map_polygon_mode(desc.rasterizer.method);
            vk::GraphicsPipelineCreateInfo {
                sType: vk::STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stageCount: stages.len() as u32,
                pStages: stages.as_ptr(),
                pVertexInputState: ptr::null(), // TODO
                pInputAssemblyState: &vk::PipelineInputAssemblyStateCreateInfo {
                    sType: vk::STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    topology: data::map_topology(desc.primitive),
                    primitiveRestartEnable: vk::FALSE,
                },
                pTessellationState: ptr::null(),
                pViewportState: ptr::null(),
                pRasterizationState: &vk::PipelineRasterizationStateCreateInfo {
                    sType: vk::STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    depthClampEnable: vk::TRUE,
                    rasterizerDiscardEnable: vk::FALSE,
                    polygonMode: polygon,
                    cullMode: data::map_cull_face(desc.rasterizer.cull_face),
                    frontFace: data::map_front_face(desc.rasterizer.front_face),
                    depthBiasEnable: if desc.rasterizer.offset.is_some() { vk::TRUE } else { vk::FALSE },
                    depthBiasConstantFactor: desc.rasterizer.offset.map_or(0.0, |off| off.1 as f32),
                    depthBiasClamp: 1.0,
                    depthBiasSlopeFactor: desc.rasterizer.offset.map_or(0.0, |off| off.0 as f32),
                    lineWidth: line_width,
                },
                pMultisampleState: ptr::null(), // TODO:
                pDepthStencilState: &vk::PipelineDepthStencilStateCreateInfo {
                    sType: vk::STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    depthTestEnable: match desc.depth_stencil {
                        Some((_, pso::DepthStencilInfo { depth: Some(_), ..} )) => vk::TRUE,
                        _ => vk::FALSE,
                    },
                    depthWriteEnable: match desc.depth_stencil {
                        Some((_, pso::DepthStencilInfo { depth: Some(state::Depth { write: true, ..}), ..} )) => vk::TRUE,
                        _ => vk::FALSE,
                    },
                    depthCompareOp: match desc.depth_stencil {
                        Some((_, pso::DepthStencilInfo { depth: Some(state::Depth { fun, ..}), ..} )) => data::map_comparison(fun),
                        _ => vk::COMPARE_OP_NEVER,
                    },
                    depthBoundsTestEnable: vk::FALSE,
                    stencilTestEnable: match desc.depth_stencil {
                        Some((_, pso::DepthStencilInfo { front: Some(_), ..} )) => vk::TRUE,
                        Some((_, pso::DepthStencilInfo { back: Some(_), ..} )) => vk::TRUE,
                        _ => vk::FALSE,
                    },
                    front: match desc.depth_stencil {
                        Some((_, pso::DepthStencilInfo { front: Some(ref s), ..} )) => data::map_stencil_side(s),
                        _ => unsafe { mem::zeroed() },
                    },
                    back: match desc.depth_stencil {
                        Some((_, pso::DepthStencilInfo { back: Some(ref s), ..} )) => data::map_stencil_side(s),
                        _ => unsafe { mem::zeroed() },
                    },
                    minDepthBounds: 0.0,
                    maxDepthBounds: 1.0,
                },
                pColorBlendState: ptr::null(), // TODO
                pDynamicState: &vk::PipelineDynamicStateCreateInfo {
                    sType: vk::STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    dynamicStateCount: 4,
                    pDynamicStates: [
                        vk::DYNAMIC_STATE_VIEWPORT,
                        vk::DYNAMIC_STATE_SCISSOR,
                        vk::DYNAMIC_STATE_BLEND_CONSTANTS,
                        vk::DYNAMIC_STATE_STENCIL_REFERENCE,
                        ].as_ptr(),
                },
                // TODO:
                layout: 0,
                renderPass: 0,
                subpass: 0,
                basePipelineHandle: 0,
                basePipelineIndex: 0,
            }
        }).collect::<Vec<_>>();

        let pipelines = {
            let (dev, vk) = self.device.get();
            let mut pipelines = Vec::with_capacity(create_infos.len());
            assert_eq!(vk::SUCCESS, unsafe {
                vk.CreateGraphicsPipelines(dev, 0, pipelines.len() as u32, create_infos.as_ptr(), ptr::null(), pipelines.as_mut_ptr())
            });
            pipelines
        };

        unimplemented!()
    }

    fn create_pipeline_cache(&mut self) -> () {

    }

    fn create_buffer_raw(&mut self, info: buffer::Info) -> Result<handle::RawBuffer<R>, buffer::CreationError> {
        let (usage, _) = data::map_usage_tiling(info.usage, info.bind);
        let native_info = vk::BufferCreateInfo {
            sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            size: info.size as vk::DeviceSize,
            usage: usage,
            sharingMode: vk::SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
        };
        let (dev, vk) = self.device.get();
        let mut buf = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateBuffer(dev, &native_info, ptr::null(), &mut buf)
        });
        let reqs = unsafe {
            let mut out = mem::zeroed();
            vk.GetBufferMemoryRequirements(dev, buf, &mut out);
            out
        };
        let mem = self.alloc(info.usage, reqs);
        assert_eq!(vk::SUCCESS, unsafe {
            vk.BindBufferMemory(dev, buf, mem, 0)
        });
        let buffer = native::Buffer {
            buffer: buf,
            memory: mem,
        };
        Ok(self.share.handles.borrow_mut().make_buffer(buffer, info))
    }

    fn create_buffer_view(&mut self) -> () {

    }

    fn create_image(&mut self, desc: texture::Info, hint: Option<core::format::ChannelType>) -> Result<handle::RawTexture<R>, texture::CreationError> {
        use core::handle::Producer;

        let (w, h, d, aa) = desc.kind.get_dimensions();
        let slices = desc.kind.get_num_slices();
        let (usage, tiling) = data::map_usage_tiling(desc.usage, desc.bind);
        let chan_type = hint.unwrap_or(core::format::ChannelType::Uint);
        let info = vk::ImageCreateInfo {
            sType: vk::STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: vk::IMAGE_CREATE_MUTABLE_FORMAT_BIT |
                (if desc.kind.is_cube() {vk::IMAGE_CREATE_CUBE_COMPATIBLE_BIT} else {0}),
            imageType: data::map_image_type(desc.kind),
            format: match data::map_format(desc.format, chan_type) {
                Some(f) => f,
                None => return Err(texture::CreationError::Format(desc.format, hint)),
            },
            extent: vk::Extent3D {
                width: w as u32,
                height: h as u32,
                depth: if slices.is_none() {d as u32} else {1},
            },
            mipLevels: desc.levels as u32,
            arrayLayers: slices.unwrap_or(1) as u32,
            samples: aa.get_num_fragments() as vk::SampleCountFlagBits,
            tiling: tiling,
            usage: usage,
            sharingMode: vk::SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            initialLayout: data::map_image_layout(desc.bind),
        };
        let (dev, vk) = self.device.get();
        let mut image = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateImage(dev, &info, ptr::null(), &mut image)
        });
        let reqs = unsafe {
            let mut out = mem::zeroed();
            vk.GetImageMemoryRequirements(dev, image, &mut out);
            out
        };
        let img = native::Image {
            image: image,
            layout: cell::Cell::new(info.initialLayout),
            memory: self.alloc(desc.usage, reqs),
        };
        assert_eq!(vk::SUCCESS, unsafe {
            vk.BindImageMemory(dev, image, img.memory, 0)
        });
        Ok(self.share.handles.borrow_mut().make_image(img, desc))
    }

    fn create_image_view(&mut self) -> () {

    }

    fn create_sampler(&mut self, info: texture::SamplerInfo) -> () {
        use core::handle::Producer;

        let (min, mag, mip, aniso) = data::map_filter(info.filter);
        let native_info = vk::SamplerCreateInfo {
            sType: vk::STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            magFilter: mag,
            minFilter: min,
            mipmapMode: mip,
            addressModeU: data::map_wrap(info.wrap_mode.0),
            addressModeV: data::map_wrap(info.wrap_mode.1),
            addressModeW: data::map_wrap(info.wrap_mode.2),
            mipLodBias: info.lod_bias.into(),
            anisotropyEnable: if aniso > 0.0 { vk::TRUE } else { vk::FALSE },
            maxAnisotropy: aniso,
            compareEnable: if info.comparison.is_some() { vk::TRUE } else { vk::FALSE },
            compareOp: data::map_comparison(info.comparison.unwrap_or(state::Comparison::Never)),
            minLod: info.lod_range.0.into(),
            maxLod: info.lod_range.1.into(),
            borderColor: match data::map_border_color(info.border) {
                Some(bc) => bc,
                None => {
                    error!("Unsupported border color {:x}", info.border.0);
                    vk::BORDER_COLOR_FLOAT_TRANSPARENT_BLACK
                }
            },
            unnormalizedCoordinates: vk::FALSE,
        };

        let (dev, vk) = self.device.get();
        let mut sampler = 0;
        assert_eq!(vk::SUCCESS, unsafe {
            vk.CreateSampler(dev, &native_info, ptr::null(), &mut sampler)
        });
        self.share.handles.borrow_mut().make_sampler(sampler, info);
    }

}