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

use {Resources, VertexCount, InstanceParams, ShaderSet};
use {buffer, format, handle, shade, texture};
use {VertexShader, GeometryShader, PixelShader};

/// Error creating either a ShaderResourceView, or UnorderedAccessView.
#[derive(Clone, PartialEq, Debug)]
pub enum ResourceViewError {
    /// The corresponding bind flag is not present in the texture.
    NoBindFlag,
    /// Selected channel type is not supported for this texture.
    Channel(format::ChannelType),
    /// Selected layer can not be viewed for this texture.
    Layer(texture::LayerError),
    /// The backend was refused for some reason.
    Unsupported,
}

pub trait Factory<R: Resources> {
    fn create_fence(&mut self, signalled: bool) -> handle::Fence<R>;
    //fn create_semaphore(&mut self) -> ();
    //fn create_event(&mut self) -> ();
    fn create_shader(&mut self, stage: shade::Stage, code: &[u8]) -> Result<handle::Shader<R>, shade::CreateShaderError>;

    /// Creates a new shader `Program` for the supplied `ShaderSet`.
    fn create_program(&mut self, shader_set: &ShaderSet<R>)
                      -> Result<handle::Program<R>, shade::CreateProgramError>;
    fn create_compute_pipelines(&mut self) -> ();
    fn create_graphics_pipelines(&mut self) -> ();
    fn create_pipeline_cache(&mut self) -> ();
    fn create_buffer_raw(&mut self, buffer::Info) -> Result<handle::RawBuffer<R>, buffer::CreationError>;
    fn create_buffer_view(&mut self) -> ();
    fn create_image(&mut self, desc: texture::Info, hint: Option<format::ChannelType>) -> Result<handle::RawTexture<R>, texture::CreationError>;
    fn create_image_view(&mut self) -> ();
    fn create_sampler(&mut self, info: texture::SamplerInfo) -> ();

    /// Compiles a `VertexShader` from source.
    fn create_shader_vertex(&mut self, code: &[u8]) -> Result<VertexShader<R>, shade::CreateShaderError> {
        self.create_shader(shade::Stage::Vertex, code).map(|s| VertexShader(s))
    }
    /// Compiles a `GeometryShader` from source.
    fn create_shader_geometry(&mut self, code: &[u8]) -> Result<GeometryShader<R>, shade::CreateShaderError> {
        self.create_shader(shade::Stage::Geometry, code).map(|s| GeometryShader(s))
    }
    /// Compiles a `PixelShader` from source. This is the same as what some APIs call a fragment
    /// shader.
    fn create_shader_pixel(&mut self, code: &[u8]) -> Result<PixelShader<R>, shade::CreateShaderError> {
        self.create_shader(shade::Stage::Pixel, code).map(|s| PixelShader(s))
    }
}