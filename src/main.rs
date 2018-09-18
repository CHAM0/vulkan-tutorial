#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;


use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::instance::Features;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano_win::VkSurfaceBuild;

use winit::WindowBuilder;
use winit::EventsLoop;



fn main() {
    // Create an instances who initializes everything 
    let extensions = vulkano_win::required_extensions();
    let instance = match Instance::new(None, &extensions, None) {
        Ok(i) => i,
        Err(err) => panic!("Couldn't build instance: {:?}", err) 
    };

    // Enumerate physical devices
    for physical_device in PhysicalDevice::enumerate(&instance) {
        println!("Available device: {}", physical_device.name());
    }

    // Choose a physical device
    let physical = match PhysicalDevice::enumerate(&instance).next() {
        Some(i) => i,
        None => panic!("No device available")
    } ;

    // Enumerate the queue families of our physical device
    for family in physical.queue_families() {
        println!("Found a queue family width {:?} queue(s)", family.queues_count());
    }

    // Choose a single queue that we will use for all of our operations
    let queue_familie = match physical.queue_families().find(|&q| q.supports_graphics()) {
        Some(i) => i,
        None => panic!("Couldn't find a graphical queue family"),
    };

    // Create a device
    let (device, mut queues) = match Device::new(physical, &Features::none(), &DeviceExtensions::none(),
            [(queue_familie, 0.5)].iter().cloned()) {
                Ok(i) => i,
                Err(err) => panic!("Failed to create device: {:?}", err),
            };

    // Get our queue 
    let queue = queues.next();


    let data = 12;
    let buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), data);

    // Create a window
    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();


    events_loop.run_forever(|event| {
        match event {
            winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => {
                winit::ControlFlow::Break
            },

            _ => winit::ControlFlow::Continue,
        }
    });
}
