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

use metal::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RawBuffer(pub *mut MTLBuffer);
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RawTexture(pub *mut MTLTexture);
unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Sampler(pub MTLSamplerState);
unsafe impl Send for Sampler {}
unsafe impl Sync for Sampler {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Rtv(pub *mut MTLTexture);
unsafe impl Send for Rtv {}
unsafe impl Sync for Rtv {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Dsv(pub *mut MTLTexture, pub Option<u16>);
unsafe impl Send for Dsv {}
unsafe impl Sync for Dsv {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Srv(pub *mut MTLTexture);
unsafe impl Send for Srv {}
unsafe impl Sync for Srv {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Uav;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Shader {
    func: MTLFunction,
}
unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Program {
    vs: MTLFunction,
    ps: MTLFunction,
}
unsafe impl Send for Program {}
unsafe impl Sync for Program {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pipeline {
    pipeline: MTLRenderPipelineState,
    depth_stencil: Option<MTLDepthStencilState>,
    winding: MTLWinding,
    cull: MTLCullMode,
    fill: MTLTriangleFillMode,
    alpha_to_one: bool,
    alpha_to_coverage: bool,
    depth_bias: i32,
    slope_scaled_depth_bias: i32,
    depth_clip: bool,
}
unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}

pub struct ShaderLibrary {
    lib: MTLLibrary,
}
unsafe impl Send for ShaderLibrary {}
unsafe impl Sync for ShaderLibrary {}

// ShaderLibrary isn't handled via Device.cleanup(). Not really an issue since it will usually
// live for the entire application lifetime and be cloned rarely.
impl Drop for ShaderLibrary {
    fn drop(&mut self) {
        unsafe { self.lib.release() };
    }
}

impl Clone for ShaderLibrary {
    fn clone(&self) -> Self {
        unsafe { self.lib.retain() };
        ShaderLibrary { lib: self.lib }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Buffer(native::Buffer, Usage, Bind);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Texture(native::Texture, Usage);