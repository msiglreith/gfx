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


use std::{cell, hash};
use core;
use Resources as R;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Shader {
}
unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Buffer {
}
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Image {
}
impl hash::Hash for Image {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
    	unimplemented!()
    }
}
unsafe impl Send for Image {}
unsafe impl Sync for Image {}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct ImageView {
}
unsafe impl Send for ImageView {}
unsafe impl Sync for ImageView {}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Pipeline {
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct PipelineLayout {
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct RenderPass {
}
