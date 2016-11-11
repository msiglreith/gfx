#[macro_use]
extern crate gfx;
extern crate gfx_core_next;
extern crate gfx_device_vulkan_next as vulkan;
extern crate env_logger;
extern crate winit;

use gfx_core_next::factory::Factory;
use gfx_core_next::{Instance, Surface, SwapChain};

pub type ColorFormat = gfx::format::Bgra8;

fn main() {
    env_logger::init().unwrap();
    let window = winit::WindowBuilder::new()
        .with_dimensions(1440, 900)
        .with_title("core_next".to_string()).build().unwrap();

    let (instance, share) = vulkan::Instance::new("next", 1, &[], &["VK_KHR_surface"]);
    let physical_device = &instance.enumerate_physical_devices()[0];
    let (device, queues) = physical_device.open_device(&instance, &["VK_KHR_swapchain"], |_| { true });

    let mut factory = vulkan::Factory::new(device.clone(), share.clone());

    let surface = vulkan::Surface::from_window(&instance, &window);
    let mut swap_chain = vulkan::SwapChain::new::<ColorFormat>(&mut factory, &queues[0], &surface, 1440, 900);

    let main_pool = queues[0].create_command_pool();
    let cmd_buffers = main_pool.create_command_buffers(16);

    let vertex_shader = factory.create_shader(include_bytes!("vert.spv"));
    let pixel_shader = factory.create_shader(include_bytes!("frag.spv"));

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