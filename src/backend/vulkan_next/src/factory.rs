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
use core::texture;
use std::slice;
use std::sync::Arc;
use std::ptr;
use std::mem;
use std::cell;
use vk;
use data;
use native;
use {Resources as R, Device, Share};

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
    fn alloc(&self, usage: memory::Usage, reqs: vk::MemoryRequirements) -> vk::DeviceMemory {
        unimplemented!()
    }
}

impl factory::Factory<R> for Factory {
    fn create_fence(&mut self) -> () {

    }

    fn create_shader(&mut self, stage: shade::Stage, code: &[u8]) -> Result<handle::Shader<R>, shade::CreateShaderError> {
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


    fn create_compute_pipelines(&mut self) -> () {

    }

    fn create_graphics_pipelines(&mut self) -> () {

    }

    fn create_pipeline_cache(&mut self) -> () {

    }

    fn create_buffer_raw(&mut self, info: buffer::Info) -> Result<handle::RawBuffer<R>, buffer::CreationError> {
        unimplemented!()
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