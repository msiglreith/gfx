#[macro_use]
extern crate gfx;
extern crate gfx_device_vulkan_next as vulkan;
extern crate env_logger;
extern crate winit;

pub type ColorFormat = gfx::format::Bgra8;

fn main() {
    env_logger::init().unwrap();
    let window = winit::WindowBuilder::new()
        .with_dimensions(1440, 900)
        .with_title("core_next".to_string()).build().unwrap();

    let instance = vulkan::Instance::new("next", 1, &[], &["VK_KHR_surface"]);
    let physical_device = &instance.physical_devices()[0];
    let (device, queues) = physical_device.open_device(&instance, &["VK_KHR_swapchain"], |_| { true });

    let surface = vulkan::Surface::new(&instance, &window);
    let swap_chain = vulkan::SwapChain::new::<ColorFormat>(&device, &instance, surface, 1440, 900);

    'main: loop {
        for event in window.poll_events() {
            match event {
                winit::Event::KeyboardInput(_, _, Some(winit::VirtualKeyCode::Escape)) |
                winit::Event::Closed => break 'main,
                _ => {},
            }
        }
    }
}