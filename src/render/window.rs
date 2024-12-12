use crate::errors::CreateWindowError;
use crate::render::context::queues::{QueueLabel, QueueRef};
use crate::render::frame_set::{FrameSet, MAX_FRAMES_IN_FLIGHT};
use crate::{Engine, VulkanContext};
use ash::prelude::VkResult;
use ash::vk;
use log::{trace, warn};
use std::sync::Arc;
use winit::window::Window;

pub struct WindowData {
    window: Window,
    vulkan_context: Arc<VulkanContext>,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    swapchain_configuration: SwapchainConfiguration,
    swapchain_resources: SwapchainResources,
    swapchain_sync_resources: SwapchainSyncResources,
    current_frame: usize,
}

pub struct SwapchainConfiguration {
    format: vk::Format,
    color_space: vk::ColorSpaceKHR,
    extent: vk::Extent2D,
}

pub struct SwapchainResources {
    images: Vec<vk::Image>,
}

pub struct SwapchainSyncResources {
    image_available: FrameSet<vk::Semaphore>,
    render_finished: FrameSet<vk::Semaphore>,
    in_flight_fences: FrameSet<vk::Fence>,
}

pub struct AcquiredImage {
    image: vk::Image,
    image_index: u32,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
    current_frame: usize,
    present_queue_family: u32,
}

impl AcquiredImage {
    pub fn image(&self) -> vk::Image {
        self.image
    }

    pub fn image_index(&self) -> u32 {
        self.image_index
    }

    pub fn image_available_semaphore(&self) -> vk::Semaphore {
        self.image_available_semaphore
    }

    pub fn render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphore
    }

    pub fn in_flight_fence(&self) -> vk::Fence {
        self.in_flight_fence
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn present_queue_family(&self) -> u32 {
        self.present_queue_family
    }
}

impl SwapchainConfiguration {
    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn color_space(&self) -> vk::ColorSpaceKHR {
        self.color_space
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

impl SwapchainResources {
    pub fn images(&self) -> &Vec<vk::Image> {
        &self.images
    }
}

impl SwapchainSyncResources {
    pub fn new(engine: &Engine) -> VkResult<Self> {
        Ok(Self {
            image_available: engine.vulkan().create_semaphores()?,
            render_finished: engine.vulkan().create_semaphores()?,
            in_flight_fences: engine.vulkan().create_fences_signaled()?,
        })
    }

    pub fn image_available(&self) -> &FrameSet<vk::Semaphore> {
        &self.image_available
    }

    pub fn render_finished(&self) -> &FrameSet<vk::Semaphore> {
        &self.render_finished
    }

    pub fn in_flight_fences(&self) -> &FrameSet<vk::Fence> {
        &self.in_flight_fences
    }
}

impl WindowData {
    pub(crate) fn new(engine: &mut Engine, window: Window) -> Result<Self, CreateWindowError> {
        let surface = engine.vulkan().create_surface(&window)?;

        let (swapchain, swapchain_configuration, swapchain_resources) =
            Self::setup_swapchain(engine.vulkan(), &window, surface.clone(), None)?;

        let swapchain_sync_resources = SwapchainSyncResources::new(engine)?;

        Ok(Self {
            window,
            vulkan_context: engine.vulkan(),
            surface,
            swapchain,
            swapchain_configuration,
            swapchain_resources,
            swapchain_sync_resources,
            current_frame: 0,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn reconfigure_swapchain(&mut self) -> VkResult<()> {
        (
            self.swapchain,
            self.swapchain_configuration,
            self.swapchain_resources,
        ) = Self::setup_swapchain(
            self.vulkan_context.clone(),
            &self.window,
            self.surface,
            Some(self.swapchain),
        )?;

        Ok(())
    }

    fn setup_swapchain(
        vulkan: Arc<VulkanContext>,
        window: &Window,
        surface: vk::SurfaceKHR,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> VkResult<(vk::SwapchainKHR, SwapchainConfiguration, SwapchainResources)> {
        vulkan.device().wait_idle()?;

        let present_modes = vulkan.query_present_modes(surface)?;
        let surface_formats = vulkan.query_surface_formats(surface)?;
        let surface_capabilities = vulkan.query_surface_capabilities(surface)?;

        // TODO: Swapchain configuration requests

        trace!("[swapchain/configuration] Available present modes:");
        present_modes
            .iter()
            .for_each(|m| trace!("[swapchain/configuration/#] - {:?}", m));

        let present_mode = present_modes
            .into_iter()
            .find(|m| m == &vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        trace!(
            "[swapchain/configuration] Selected present mode: {:?}",
            present_mode
        );

        trace!("[swapchain/configuration] Available surface formats:");
        surface_formats.iter().for_each(|m| {
            trace!(
                "[swapchain/configuration/#] - (format: {:?}, color_space: {:?})",
                m.format, m.color_space
            )
        });

        let surface_format = surface_formats
            .iter()
            .cloned()
            .filter(|f| f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
            .unwrap_or(surface_formats[0]);

        trace!(
            "[swapchain/configuration] Selected surface format: {:?}",
            surface_format
        );

        let min_image_count = if surface_capabilities.max_image_count > 0 {
            surface_capabilities
                .max_image_count
                .min(surface_capabilities.min_image_count + 1)
        } else {
            surface_capabilities.min_image_count + 1
        };

        trace!(
            "[swapchain/configuration] Selected swapchain min image count: {:?}",
            min_image_count
        );

        let extent = if surface_capabilities.current_extent.width == u32::MAX {
            vk::Extent2D {
                width: window.inner_size().width.clamp(
                    surface_capabilities.max_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: window.inner_size().height.clamp(
                    surface_capabilities.max_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        } else {
            surface_capabilities.current_extent
        };

        trace!(
            "[swapchain/configuration] Selected swapchain extent: {:?}",
            extent
        );

        let swapchain = unsafe {
            vulkan.device().loader().swapchain().create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(surface)
                    .present_mode(present_mode)
                    .min_image_count(min_image_count)
                    .image_format(surface_format.format)
                    .image_color_space(surface_format.color_space)
                    .image_usage(
                        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                    )
                    .image_array_layers(1)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .image_extent(extent)
                    .clipped(true)
                    .pre_transform(surface_capabilities.current_transform)
                    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .old_swapchain(old_swapchain.unwrap_or(vk::SwapchainKHR::null())),
                None,
            )
        }?;

        let images = unsafe {
            vulkan
                .device()
                .loader()
                .swapchain()
                .get_swapchain_images(swapchain)
        }?;

        trace!(
            "[window/swapchain] Created swapchain with {:?} images.",
            images.len()
        );

        let cfg = SwapchainConfiguration {
            format: surface_format.format,
            color_space: surface_format.color_space,
            extent,
        };

        let res = SwapchainResources { images };

        Ok((swapchain, cfg, res))
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn swapchain_configuration(&self) -> &SwapchainConfiguration {
        &self.swapchain_configuration
    }

    pub fn swapchain_resources(&self) -> &SwapchainResources {
        &self.swapchain_resources
    }

    pub fn swapchain_sync_resources(&self) -> &SwapchainSyncResources {
        &self.swapchain_sync_resources
    }

    fn acquire_image(&self, prqf: u32) -> VkResult<(AcquiredImage, bool)> {
        let in_flight_fence = self.swapchain_sync_resources.in_flight_fences[self.current_frame];
        self.vulkan_context.wait_for_fence(in_flight_fence)?;

        let image_available_semaphore =
            self.swapchain_sync_resources.image_available[self.current_frame];
        let (image_index, suboptimal) = unsafe {
            self.vulkan_context
                .device()
                .loader()
                .swapchain()
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    image_available_semaphore,
                    vk::Fence::null(),
                )
        }?;

        let image = self.swapchain_resources.images()[image_index as usize];

        self.vulkan_context
            .reset_fence(self.swapchain_sync_resources.in_flight_fences[self.current_frame])?;

        Ok((
            AcquiredImage {
                image,
                image_index: image_index.clone(),
                current_frame: self.current_frame,
                image_available_semaphore,
                render_finished_semaphore: self.swapchain_sync_resources.render_finished
                    [self.current_frame],
                in_flight_fence,
                present_queue_family: prqf,
            },
            suboptimal,
        ))
    }

    fn present_image(&mut self, image: AcquiredImage, prqref: QueueRef) -> VkResult<bool> {
        // TODO: turn this expect into an error
        let suboptimal = unsafe {
            self.vulkan_context
                .device()
                .loader()
                .swapchain()
                .queue_present(
                    self.vulkan_context
                        .device()
                        .get_queue(prqref)
                        .expect("No presentation queue"),
                    &vk::PresentInfoKHR::default()
                        .swapchains(&[self.swapchain])
                        .wait_semaphores(&[image.render_finished_semaphore])
                        .image_indices(&[image.image_index]),
                )
        }?;

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;


        Ok(suboptimal)
    }

    fn render_frame_inner<F: FnOnce(&Self, &AcquiredImage) -> VkResult<()>>(&mut self, f: F) -> VkResult<bool> {
        let prqref = self.vulkan_context.device().get_labeled_queue_ref(QueueLabel::Presentation).expect("No presentation queue");

        let (acquired_image, suboptimal) = self.acquire_image(prqref.family)?;

        f(self, &acquired_image)?;

        self.present_image(acquired_image, prqref)?;

        Ok(suboptimal)
    }

    pub fn render_frame<F: FnOnce(&Self, &AcquiredImage) -> VkResult<()>>(&mut self, f: F) -> VkResult<()> {
        match self.render_frame_inner(f) {
            Ok(false) => Ok(()),
            Ok(true) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                warn!("Swapchain configuration suboptimal");
                self.reconfigure_swapchain()
            },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                warn!("Swapchain configuration out of date");
                self.reconfigure_swapchain()
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for WindowData {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_context
                .device()
                .loader()
                .swapchain()
                .destroy_swapchain(self.swapchain, None);

            self.vulkan_context
                .instance()
                .loader()
                .surface()
                .destroy_surface(self.surface, None);
        }
    }
}
