use erupt::{
    extensions::{khr_surface, khr_swapchain},
    utils::surface,
    utils::allocator::{Allocator, AllocatorCreateInfo, MemoryTypeFinder},
    vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader,
    cstr,
};
use anyhow::{Result, format_err};
use std::ffi::CString;

fn main() -> Result<()> {
    // Entry
    let entry = EntryLoader::new()?;

    // Instance
    let application_name = CString::new("Minimal example")?;
    let engine_name = CString::new("Minimal example")?;
    let app_info = vk::ApplicationInfoBuilder::new()
        .application_name(&application_name)
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    // Instance and device layers and extensions
    let mut instance_layers = Vec::new();
    let mut instance_extensions = Vec::new();
    let mut device_layers = Vec::new();
    let mut device_extensions = Vec::new();

    // Vulkan layers and extensions
    const LAYER_KHRONOS_VALIDATION: *const i8 = cstr!("VK_LAYER_KHRONOS_validation");

    if cfg!(debug_assertions) {
        instance_extensions
            .push(erupt::extensions::ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME);
        instance_layers.push(LAYER_KHRONOS_VALIDATION);
        device_layers.push(LAYER_KHRONOS_VALIDATION);
    }

    // Instance creation
    let create_info = vk::InstanceCreateInfoBuilder::new()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions)
        .enabled_layer_names(&instance_layers);

    let instance = InstanceLoader::new(&entry, &create_info, None)?;

    // Queue family and physical_device device selection
    let (queue_family_index, physical_device) = query_hardware(&instance)?;

    // Create device and queue
    let create_info = [vk::DeviceQueueCreateInfoBuilder::new()
        .queue_family_index(queue_family_index)
        .queue_priorities(&[1.0])];
    let physical_device_features = vk::PhysicalDeviceFeaturesBuilder::new();
    let create_info = vk::DeviceCreateInfoBuilder::new()
        .queue_create_infos(&create_info)
        .enabled_features(&physical_device_features)
        .enabled_extension_names(&device_extensions)
        .enabled_layer_names(&device_layers);

    let device = DeviceLoader::new(&instance, physical_device, &create_info, None)?;
    let queue = unsafe { device.get_device_queue(queue_family_index, 0, None) };

    // Device memory allocator
    let mut allocator = Allocator::new(
        &instance,
        physical_device,
        AllocatorCreateInfo::default(),
    )
    .result()?;

    // THE GOOD BIT
    let create_info = vk::BufferCreateInfoBuilder::new()
        .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .size(128u64);

    let buffer_a = unsafe { device.create_buffer(&create_info, None, None) }.result()?;
    let memory_a = allocator
        .allocate(
            &device,
            buffer_a,
            MemoryTypeFinder::dynamic(),
        )
        .result()?;
    let map_a = memory_a.map(&device, ..).result()?;
    //map_a.unmap(&device).result()?;

    let buffer_b = unsafe { device.create_buffer(&create_info, None, None) }.result()?;
    let memory_b = allocator
        .allocate(
            &device,
            buffer_b,
            MemoryTypeFinder::dynamic(),
        )
        .result()?;
    let map_b = memory_b.map(&device, ..).result()?;

    Ok(())
}

fn query_hardware(instance: &InstanceLoader) -> Result<(u32, vk::PhysicalDevice)> {
    let physical_devices = unsafe { instance.enumerate_physical_devices(None) }.result()?;
    for device in physical_devices {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device, None) };
        for (idx, properties) in queue_families.into_iter().enumerate() {
            if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                return Ok((idx as u32, device));
            }
        }
    }
    Err(format_err!("No suitable device found"))
}
