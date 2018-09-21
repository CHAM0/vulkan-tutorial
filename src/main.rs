#[macro_use] extern crate vulkano;
#[macro_use] extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;
extern crate image;            

use vulkano::instance::{ Instance, InstanceExtensions, PhysicalDevice, Features };
use vulkano::device::{ Device, DeviceExtensions };
use vulkano::buffer::{ BufferUsage, CpuAccessibleBuffer };
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::command_buffer::{ AutoCommandBufferBuilder, CommandBuffer };
use vulkano::sync::GpuFuture;
use vulkano::format::{ Format, ClearValue };
use vulkano::image::{ Dimensions, StorageImage };

use vulkano_win::VkSurfaceBuild;

use winit::WindowBuilder;
use winit::EventsLoop;

use std::sync::Arc;

use image::{ ImageBuffer, Rgba };


mod cs {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "src/shaders/operation.glsl"]
    #[allow(dead_code)]

    struct Dummy;
}

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
    let queue = queues.next().expect("Failed to get our queue");


    // Introduction to compute operations
    let data_iter = 0 .. 65536;
    let data_buffer = match CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter) {
        Ok(i) => i,
        Err(err) => panic!("Failed to create a buffer: {:?}", err)
    };

    // Create shader module
    let shader = match cs::Shader::load(device.clone()) {
        Ok(i) => i,
        Err(err) => panic!("Failed to create a shader module: {:?}", err),
    };

    // Create a compute pipeline
    let compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
        .expect("failed to create compute pipeline"));

    // Create a descriptor set
    let set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
        .add_buffer(data_buffer.clone()).unwrap()
        .build().unwrap()
    );

    // Create a command buffer 
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
        .dispatch([1024,1,1], compute_pipeline.clone(), set.clone(), ()).unwrap()
        .build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();

    finished.then_signal_fence_and_flush().unwrap()
        .wait(None).unwrap();

    let content = data_buffer.read().unwrap();
        for (n, val) in content.iter().enumerate() {
            assert_eq!(*val, n as u32 * 12);
    }
    println!("Everything succeeded!");



    // Image creation
    let image = StorageImage::new(device.clone(), Dimensions::Dim2d {width: 1024, height: 1024},
        Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();

    let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
        (0..1024 * 1024 * 4).map(|_| 0u8))
        .expect("Failed to create buffer");
    
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
        .clear_color_image(image.clone(), ClearValue::Float([0.3, 0.3, 0.3, 1.0])).unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone()).unwrap()
        .build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap()
        .wait(None).unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();

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
