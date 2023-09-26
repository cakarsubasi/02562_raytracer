use wgpu::{Adapter, AdapterInfo, Device, Instance, Queue};

// A convenience wrapper for interfacing with the GPU.
pub struct GPUHandles {
    pub queue: Queue,
    pub adapter: Adapter,
    pub instance: Instance,
    pub device: Device,
}

impl GPUHandles {
    pub fn new() -> Self {
        let instance: Instance = get_instance();

        // You might want to change this to prefer a certain backend or a high power GPU.
        let adapter: Adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                ..Default::default()
            }))
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        // If you want to run with a webgl backend, you
        // can set the limits to one of the downlevels
        let (device, queue): (Device, Queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ))
        .expect("Failed to create device");

        GPUHandles {
            queue,
            adapter,
            instance,
            device,
        }
    }
}

impl Default for GPUHandles {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_instance() -> wgpu::Instance {
    // Backends::all => Vulkan + Metal + DX12 + Browser
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::BROWSER_WEBGPU,
        dx12_shader_compiler: Default::default(),
    });
    instance
}

// Checks whether the system has a findable adapter (GPU).
// Returns false if no adapter is found.
pub fn self_test() -> bool {
    eprintln!("Performing self test to check system for compatibility.");
    let instance: Instance = get_instance();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter_option: Option<Adapter> =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()));

    // Handle whether we find a GPU or not.
    match adapter_option {
        Some(adapter) => {
            let info: AdapterInfo = adapter.get_info();
            println!("Found GPU: {:?}", info);
            true
        }
        None => {
            println!("Failed to find a usable GPU. This framework will only run CPU code.");
            false
        }
    }
}
