#[macro_use]
extern crate gfx;
extern crate gfx_core_next;
extern crate gfx_device_dx12_next as dx12;
extern crate env_logger;
extern crate winit;

use gfx_core_next::factory::Factory;
use gfx_core_next::Instance;
use gfx_core_next::{Surface, SwapChain, PhysicalDevice};

use std::sync::Arc;

pub type ColorFormat = gfx::format::Rgba8;

fn main() {
    env_logger::init().unwrap();
    let window = winit::WindowBuilder::new()
        .with_dimensions(1440, 900)
        .with_title("next_dx12".to_string()).build().unwrap();

    println!("create instance");

    let mut instance = Arc::new(dx12::Instance::new());
    let devices = instance.enumerate_physical_devices();
    println!("instance done");

    let (device, queues) = devices[0].open_device();
    let mut factory = dx12::Factory { }; //vulkan::Factory::new(device.clone(), share.clone());

    let surface = dx12::Surface::from_window(&instance, &window);
    let mut swap_chain = dx12::SwapChain::new::<ColorFormat>(&mut factory, &queues[0], &surface, 1440, 900);

    /*
    let main_pool = queues[0].create_command_pool();
    let cmd_buffers = main_pool.create_command_buffers(16);

    let vertex_shader = factory.create_shader(include_bytes!("vert.spv"));
    let pixel_shader = factory.create_shader(include_bytes!("frag.spv"));
    */

    'main: loop {
        for event in window.poll_events() {
            match event {
                winit::Event::KeyboardInput(_, _, Some(winit::VirtualKeyCode::Escape)) |
                winit::Event::Closed => break 'main,
                _ => {},
            }
        }

        swap_chain.present();
    }
}