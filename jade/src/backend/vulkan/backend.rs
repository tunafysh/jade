use crate::backend::{
    backend::Backend,
    vulkan::{
        helpers::{
            check_hdr, create_instance, create_logical_device, create_surface,
            create_swapchain_config, init_swapchain, pick_physical_device_heuristic,
        },
        types::SwapchainConfig,
    },
    window::Window,
};
use anyhow::Result;
use ash::{
    Device, Instance,
    khr::{surface::Instance as SurfaceInstance, swapchain::Device as SwapchainLoader},
    vk::{PhysicalDevice, Queue, SurfaceKHR, SwapchainKHR},
};
use tracing::instrument;

pub struct VulkanBackend {
    window: Window,
    instance: Instance,
    surface_loader: ash::khr::surface::Instance,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
    logical_device: Device,
    graphics_queue: Queue,
    present_queue: Queue,
    hdr: bool,
    vsync: bool,
    swapchain_config: SwapchainConfig,
    swapchain_loader: SwapchainLoader,
    swapchain: SwapchainKHR,
}

impl VulkanBackend {
    #[instrument(name = "Vulkan - Recreate Swapchain Config", skip_all)]
    pub fn recreate_swapchain_config(&mut self) -> Result<()> {
        let config = create_swapchain_config(
            self.physical_device,
            &self.surface_loader,
            &mut self.surface,
            self.hdr,
            self.vsync,
        )?;
        self.swapchain_config = config;
        Ok(())
    }

    #[instrument(name = "Vulkan - Recreate Swapchain", skip_all)]
    pub fn recreate_swapchain(&mut self) -> Result<()> {
        let (swapchain_loader, swapchain) = init_swapchain(
            &mut self.instance,
            &self.logical_device,
            &mut self.surface,
            self.swapchain_config,
        )?;

        self.swapchain_loader = swapchain_loader;
        self.swapchain = swapchain;

        Ok(())
    }

    #[instrument(name = "Vulkan - Enable HDR", skip_all)]
    pub fn enable_hdr(&mut self) -> Result<()> {
        self.hdr = true;
        Ok(())
    }

    #[instrument(name = "Vulkan - Disable HDR", skip_all)]
    pub fn disable_hdr(&mut self) -> Result<()> {
        self.hdr = false;
        Ok(())
    }

    #[instrument(name = "Vulkan - Enable VSync", skip_all)]
    pub fn enable_vsync(&mut self) -> Result<()> {
        self.vsync = true;
        Ok(())
    }

    #[instrument(name = "Vulkan - Disable VSync", skip_all)]
    pub fn disable_vsync(&mut self) -> Result<()> {
        self.vsync = false;
        Ok(())
    }
}

impl Backend for VulkanBackend {
    #[instrument(name = "Vulkan - Initialize Backend", skip_all)]
    fn init(mut window: Window) -> Result<Self> {
        let (entry, mut instance) = create_instance(&mut window)?;
        let mut surface = create_surface(&mut window, &mut instance)?;
        let surface_loader = SurfaceInstance::new(&entry, &instance);

        let physical_device =
            pick_physical_device_heuristic(&mut instance, &surface_loader, &mut surface)?;

        let is_hdr_supported = check_hdr(physical_device, &surface_loader, &mut surface)?;

        let (logical_device, queue_families) = create_logical_device(
            &mut instance,
            physical_device,
            &surface_loader,
            &mut surface,
        )?;

        let present_family = queue_families
            .present
            .ok_or_else(|| anyhow::anyhow!("No present queue family"))?;

        let graphics_family = queue_families
            .graphics
            .ok_or_else(|| anyhow::anyhow!("No present queue family"))?;

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_family, 0) };

        let present_queue = unsafe { logical_device.get_device_queue(present_family, 0) };

        let swapchain_config = create_swapchain_config(
            physical_device,
            &surface_loader,
            &mut surface,
            is_hdr_supported,
            true, // vsync enabled by default
        )?;

        let (swapchain_loader, swapchain) = init_swapchain(
            &mut instance,
            &logical_device,
            &mut surface,
            swapchain_config,
        )?;

        Ok(Self {
            window,
            instance,
            surface_loader,
            surface,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            hdr: is_hdr_supported,
            vsync: true,
            swapchain_config,
            swapchain_loader,
            swapchain,
        })
    }

    fn add_shader(&mut self) {
        todo!()
    }

    fn begin_frame(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn draw(&mut self, _command: &crate::backend::draw_command::DrawCommand) -> anyhow::Result<()> {
        todo!()
    }

    fn end_frame(&mut self) {
        todo!()
    }

    fn window(&mut self) -> &mut crate::backend::window::Window {
        &mut self.window
    }
}
