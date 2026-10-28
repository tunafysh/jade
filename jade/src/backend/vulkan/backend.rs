use std::collections::HashMap;

use crate::{
    backend::{
        backend::Backend,
        vulkan::{
            helpers::{
                allocate_command_buffers, check_hdr, create_command_pool, create_graphics_pipeline,
                create_instance, create_logical_device, create_semaphore, create_shader_module,
                create_signaled_fence, create_surface, create_swapchain, create_swapchain_config,
                create_swapchain_image_views, pick_physical_device_heuristic,
            },
            types::{GraphicsPipeline, PipelineConfig, SwapchainConfig},
        },
        window::Window,
    },
    drop_error,
    shading::{reflection::ShaderReflection, shader::Shader},
};
use anyhow::{Result, anyhow};
use ash::{
    Device, Instance, khr::{surface::Instance as SurfaceInstance, swapchain::Device as SwapchainLoader}, vk::{
        AccessFlags, AttachmentLoadOp, AttachmentStoreOp, ClearColorValue, ClearValue, CommandBuffer, CommandBufferBeginInfo, CommandBufferResetFlags, CommandPool, DependencyFlags, Extent2D, Fence, Handle, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageView, Offset2D, PhysicalDevice, PipelineBindPoint, PipelineStageFlags, PresentInfoKHR, QUEUE_FAMILY_IGNORED, Queue, Rect2D, RenderingAttachmentInfo, RenderingInfo, Semaphore, ShaderModule, SubmitInfo, SurfaceKHR, SwapchainKHR,
    },
};
use tracing::{info, instrument};

const TRIANGLE_SPIRV: &[u8] = include_bytes!("../../../../assets/triangle.spv");

pub struct VulkanBackend {
    window: Window,
    instance: Instance,
    surface_loader: ash::khr::surface::Instance,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
    logical_device: Device,
    graphics_queue: Queue,
    present_queue: Queue,
    shader_cache: HashMap<String, ShaderModule>,
    hdr: bool,
    vsync: bool,
    swapchain_config: SwapchainConfig,
    swapchain_loader: SwapchainLoader,
    swapchain: SwapchainKHR,
    swapchain_images: Vec<Image>,
    swapchain_image_layouts: Vec<ImageLayout>,
    image_views: Vec<ImageView>,
    pipeline: GraphicsPipeline,
    command_pool: CommandPool,
    command_buffers: Vec<CommandBuffer>,
    image_available: Semaphore,
    render_finished: Semaphore,
    in_flight: Fence,
    current_image_index: Option<u32>,
}

impl VulkanBackend {
    #[instrument(name = "Vulkan - Recreate Swapchain Config", skip_all)]
    pub fn recreate_swapchain_config(&mut self) -> Result<()> {
        info!(
            width = self.window.width(),
            height = self.window.height(),
            "Recreating Vulkan swapchain configuration"
        );
        let config = create_swapchain_config(
            self.physical_device,
            &self.surface_loader,
            &mut self.surface,
            self.hdr,
            self.vsync,
            {
                let size = self.window.size();
                Extent2D {
                    width: size.width,
                    height: size.height,
                }
            },
        )?;
        self.swapchain_config = config;
        info!(
            width = config.image_extent.width,
            height = config.image_extent.height,
            "Vulkan swapchain configuration updated"
        );
        Ok(())
    }

    #[instrument(name = "Vulkan - Recreate Swapchain", skip_all)]
    pub fn recreate_swapchain(&mut self) -> Result<()> {
        info!(
            old_swapchain = self.swapchain.as_raw(),
            "Waiting for device before recreating Vulkan swapchain"
        );
        unsafe {
            self.logical_device.device_wait_idle()?;
        }

        let (swapchain_loader, swapchain) = create_swapchain(
            &mut self.instance,
            &self.logical_device,
            &mut self.surface,
            self.swapchain_config,
        )?;

        let image_views = create_swapchain_image_views(
            &self.logical_device,
            &swapchain_loader,
            swapchain,
            self.swapchain_config.image_format,
        )?;
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        info!(
            new_swapchain = swapchain.as_raw(),
            image_view_count = image_views.len(),
            "New Vulkan swapchain resources created"
        );

        // Keep the old resources so we can destroy them.
        let old_image_views = std::mem::replace(&mut self.image_views, image_views);
        let old_swapchain = std::mem::replace(&mut self.swapchain, swapchain);
        let old_swapchain_loader = std::mem::replace(&mut self.swapchain_loader, swapchain_loader);
        self.swapchain_images = swapchain_images;
        self.swapchain_image_layouts = vec![ImageLayout::UNDEFINED; self.swapchain_images.len()];

        // Destroy the old image views before their old swapchain.
        unsafe {
            for image_view in old_image_views {
                self.logical_device.destroy_image_view(image_view, None);
            }

            old_swapchain_loader.destroy_swapchain(old_swapchain, None);
        }

        info!(
            swapchain = self.swapchain.as_raw(),
            "Vulkan swapchain recreation complete"
        );

        Ok(())
    }

    #[instrument(name = "Vulkan - Enable HDR", skip_all)]
    pub fn enable_hdr(&mut self) -> Result<()> {
        info!("Enabling Vulkan HDR");
        self.hdr = true;
        Ok(())
    }

    #[instrument(name = "Vulkan - Disable HDR", skip_all)]
    pub fn disable_hdr(&mut self) -> Result<()> {
        info!("Disabling Vulkan HDR");
        self.hdr = false;
        Ok(())
    }

    #[instrument(name = "Vulkan - Enable VSync", skip_all)]
    pub fn enable_vsync(&mut self) -> Result<()> {
        info!("Enabling Vulkan VSync");
        self.vsync = true;
        Ok(())
    }

    #[instrument(name = "Vulkan - Disable VSync", skip_all)]
    pub fn disable_vsync(&mut self) -> Result<()> {
        info!("Disabling Vulkan VSync");
        self.vsync = false;
        Ok(())
    }
}

impl Backend for VulkanBackend {
    #[instrument(name = "Vulkan - Initialize Backend", skip_all)]
    fn init(mut window: Window) -> Result<Self> {
        info!(
            width = window.width(),
            height = window.height(),
            "Starting Vulkan backend initialization"
        );
        let (entry, mut instance) = create_instance(&mut window)?;
        info!("Vulkan instance created");
        let mut surface = create_surface(&mut window, &entry, &mut instance)?;
        info!(surface = surface.as_raw(), "Vulkan surface created");
        let surface_loader = SurfaceInstance::new(&entry, &instance);

        let physical_device =
            pick_physical_device_heuristic(&mut instance, &surface_loader, surface)?;
        info!(
            physical_device = physical_device.as_raw(),
            "Vulkan physical device selected"
        );

        let is_hdr_supported = check_hdr(physical_device, &surface_loader, &mut surface)?;

        let (logical_device, queue_families) = create_logical_device(
            &mut instance,
            physical_device,
            &surface_loader,
            &mut surface,
        )?;
        info!(graphics_family = ?queue_families.graphics, present_family = ?queue_families.present, "Vulkan logical device created");

        let present_family = queue_families
            .present
            .ok_or_else(|| anyhow::anyhow!("No present queue family"))?;

        let graphics_family = queue_families
            .graphics
            .ok_or_else(|| anyhow::anyhow!("No graphics queue family"))?;

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_family, 0) };

        let present_queue = unsafe { logical_device.get_device_queue(present_family, 0) };
        info!(
            graphics_queue = graphics_queue.as_raw(),
            present_queue = present_queue.as_raw(),
            "Vulkan queues acquired"
        );

        let swapchain_config = create_swapchain_config(
            physical_device,
            &surface_loader,
            &mut surface,
            is_hdr_supported,
            true, // vsync enabled by default
            {
                let size = window.size();
                Extent2D {
                    width: size.width,
                    height: size.height,
                }
            },
        )?;
        info!(
            width = swapchain_config.image_extent.width,
            height = swapchain_config.image_extent.height,
            format = swapchain_config.image_format.as_raw(),
            "Vulkan swapchain configuration created"
        );

        let (swapchain_loader, swapchain) = create_swapchain(
            &mut instance,
            &logical_device,
            &mut surface,
            swapchain_config,
        )?;
        info!(swapchain = swapchain.as_raw(), "Vulkan swapchain created");

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let views = create_swapchain_image_views(
            &logical_device,
            &swapchain_loader,
            swapchain,
            swapchain_config.image_format,
        )?;
        info!(
            image_view_count = views.len(),
            "Vulkan swapchain image views created"
        );

        let mut shader_cache = HashMap::new();
        let shader_module = create_shader_module(&logical_device, TRIANGLE_SPIRV)?;
        info!(
            shader_module = shader_module.as_raw(),
            byte_count = TRIANGLE_SPIRV.len(),
            "Vulkan shader module created"
        );
        let pipeline_config = PipelineConfig::default();

        let pipeline = create_graphics_pipeline(
            &logical_device,
            shader_module,
            pipeline_config,
            swapchain_config,
        )?;
        info!(
            pipeline = pipeline.handle.as_raw(),
            pipeline_layout = pipeline.layout.as_raw(),
            "Vulkan graphics pipeline created"
        );

        shader_cache.insert("triangle".to_string(), shader_module);

        let command_pool = create_command_pool(&logical_device, graphics_family)?;
        let command_buffers = allocate_command_buffers(&logical_device, command_pool)?;
        info!(
            command_pool = command_pool.as_raw(),
            command_buffer_count = command_buffers.len(),
            "Vulkan command resources created"
        );

        let image_available = create_semaphore(&logical_device)?;
        let render_finished = create_semaphore(&logical_device)?;

        let in_flight = create_signaled_fence(&logical_device)?;
        info!(
            image_available = image_available.as_raw(),
            render_finished = render_finished.as_raw(),
            in_flight = in_flight.as_raw(),
            "Vulkan synchronization resources created"
        );
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
            swapchain_images,
            swapchain_image_layouts: vec![ImageLayout::UNDEFINED; views.len()],
            image_views: views,
            shader_cache,
            pipeline,
            command_pool,
            command_buffers,
            render_finished,
            image_available,
            in_flight,
            current_image_index: None,
        })
    }

    fn add_shader(&mut self, _shader: Shader, _reflection: ShaderReflection) {
        todo!()
    }

    #[instrument(name = "Vulkan - Begin Frame", skip_all)]
    fn begin_frame(&mut self) -> anyhow::Result<()> {
        info!(
            swapchain = self.swapchain.as_raw(),
            "Beginning Vulkan frame"
        );
        unsafe {
            self.logical_device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)?;
        }

        let (image_index, _is_suboptimal) = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available,
                Fence::null(),
            )
        }?;
        info!(image_index, "Vulkan swapchain image acquired");

        unsafe {
            self.logical_device
                .reset_command_buffer(self.command_buffers[0], CommandBufferResetFlags::empty())?
        };

        let begin_info = CommandBufferBeginInfo::default();

        unsafe {
            self.logical_device
                .begin_command_buffer(self.command_buffers[0], &begin_info)?;
        }

        unsafe { self.logical_device.reset_fences(&[self.in_flight])? };

        self.current_image_index = Some(image_index);
        info!(
            command_buffer = self.command_buffers[0].as_raw(),
            "Vulkan command buffer recording started"
        );

        let image = self.swapchain_images[image_index as usize];
        let old_layout = self.swapchain_image_layouts[image_index as usize];
        let barrier = ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .src_access_mask(AccessFlags::empty())
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
            .src_queue_family_index(QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(
                ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        unsafe {
            self.logical_device.cmd_pipeline_barrier(
                self.command_buffers[0],
                if old_layout == ImageLayout::UNDEFINED {
                    PipelineStageFlags::TOP_OF_PIPE
                } else {
                    PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                },
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        let color_attachment = RenderingAttachmentInfo::default()
            .image_view(self.image_views[image_index as usize])
            .image_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .clear_value(ClearValue {
                color: ClearColorValue {
                    float32: [0.05, 0.08, 0.15, 1.0],
                },
            });

        let color_attachments = [color_attachment];

        let rendering_info = RenderingInfo::default()
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: self.swapchain_config.image_extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        unsafe {
            self.logical_device
                .cmd_begin_rendering(self.command_buffers[0], &rendering_info);
        }
            self.swapchain_image_layouts[image_index as usize] = ImageLayout::COLOR_ATTACHMENT_OPTIMAL;

        Ok(())
    }

    #[instrument(name = "Vulkan - Draw", skip_all)]
    fn draw(&mut self) -> anyhow::Result<()> {
        info!(
            command_buffer = self.command_buffers[0].as_raw(),
            pipeline = self.pipeline.handle.as_raw(),
            "Recording Vulkan draw command"
        );
        unsafe {
            self.logical_device.cmd_bind_pipeline(
                self.command_buffers[0],
                PipelineBindPoint::GRAPHICS,
                self.pipeline.handle,
            );

            self.logical_device.cmd_set_viewport(
                self.command_buffers[0],
                0,
                &[ash::vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: self.swapchain_config.image_extent.width as f32,
                    height: self.swapchain_config.image_extent.height as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );
            self.logical_device.cmd_set_scissor(
                self.command_buffers[0],
                0,
                &[ash::vk::Rect2D {
                    offset: Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_config.image_extent,
                }],
            );

            self.logical_device
                .cmd_draw(self.command_buffers[0], 3, 1, 0, 0);
        }

        Ok(())
    }

    #[instrument(name = "Vulkan - End Frame", skip_all)]
    fn end_frame(&mut self) -> Result<()> {
        info!(image_index = ?self.current_image_index, "Ending Vulkan frame");
        let wait_semaphores = [self.image_available];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffers[0]];
        let signal_semaphores = [self.render_finished];

        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        let image_index = self
            .current_image_index
            .ok_or(anyhow!("No current image."))?;

        let wait_semaphores = [self.render_finished];
        let swapchains = [self.swapchain];
        let image_indices = [image_index];

        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let image = self.swapchain_images[image_index as usize];

        unsafe {
            self.logical_device
                .cmd_end_rendering(self.command_buffers[0]);

            let barrier = ImageMemoryBarrier::default()
                .old_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(ImageLayout::PRESENT_SRC_KHR)
                .src_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(AccessFlags::empty())
                .src_queue_family_index(QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(
                    ImageSubresourceRange::default()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

            self.logical_device.cmd_pipeline_barrier(
                self.command_buffers[0],
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                PipelineStageFlags::BOTTOM_OF_PIPE,
                DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        unsafe {
            self.logical_device
                .end_command_buffer(self.command_buffers[0])?;

            self.logical_device.queue_submit(
                self.graphics_queue,
                &[submit_info],
                self.in_flight,
            )?;

            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)?;
        }
        self.swapchain_image_layouts[image_index as usize] = ImageLayout::PRESENT_SRC_KHR;

        info!(image_index, "Vulkan frame submitted and presented");

        Ok(())
    }

    fn window(&mut self) -> &mut crate::backend::window::Window {
        &mut self.window
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        info!(
            swapchain = self.swapchain.as_raw(),
            "Destroying Vulkan backend"
        );
        unsafe {
            drop_error!(self.logical_device.device_wait_idle());
            self.logical_device.destroy_fence(self.in_flight, None);
            self.logical_device
                .destroy_semaphore(self.image_available, None);
            self.logical_device
                .destroy_semaphore(self.render_finished, None);
            self.logical_device
                .destroy_pipeline(self.pipeline.handle, None);
            self.logical_device
                .destroy_pipeline_layout(self.pipeline.layout, None);

            for shader in self.shader_cache.values() {
                self.logical_device.destroy_shader_module(*shader, None);
            }

            for image_view in &self.image_views {
                self.logical_device.destroy_image_view(*image_view, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.logical_device
                .destroy_command_pool(self.command_pool, None);
            self.logical_device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
        info!("Vulkan backend destroyed");
    }
}
