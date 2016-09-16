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

pub trait Factory {
    fn create_fence(&mut self) -> ();
    fn create_semaphore(&mut self) -> ();
    fn create_event(&mut self) -> ();
    fn create_shader(&mut self) -> ();
    fn create_compute_pipelines(&mut self) -> ();
    fn create_graphics_pipelines(&mut self) -> ();
    fn create_pipeline_cache(&mut self) -> ();
    fn create_buffer(&mut self) -> ();
    fn create_buffer_view(&mut self) -> ();
    fn create_image(&mut self) -> ();
    fn create_image_view(&mut self) -> ();
    fn create_sampler(&mut self) -> ();
}