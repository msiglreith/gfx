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

use Resources;
use {buffer, handle, shade};
use {VertexShader, GeometryShader, PixelShader};

pub trait Factory<R: Resources> {
    fn create_fence(&mut self) -> ();
    //fn create_semaphore(&mut self) -> ();
    //fn create_event(&mut self) -> ();
    fn create_shader(&mut self, stage: shade::Stage, code: &[u8]) -> Result<handle::Shader<R>, shade::CreateShaderError>;

    fn create_compute_pipelines(&mut self) -> ();
    fn create_graphics_pipelines(&mut self) -> ();
    fn create_pipeline_cache(&mut self) -> ();
    fn create_buffer_raw(&mut self, buffer::Info) -> Result<handle::RawBuffer<R>, buffer::CreationError>;
    fn create_buffer_view(&mut self) -> ();
    fn create_image(&mut self) -> ();
    fn create_image_view(&mut self) -> ();
    fn create_sampler(&mut self) -> ();

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