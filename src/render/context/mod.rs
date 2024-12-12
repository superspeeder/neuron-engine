pub mod device;
pub mod instance;
pub mod platform;
pub mod queues;
pub mod command_pool;

use crate::errors::CreateSurfaceError;
use crate::render::context::device::Device;
use crate::render::context::instance::Instance;
use crate::render::frame_set::FrameSet;
use crate::EngineCallbackHandler;
use ash::prelude::VkResult;
use ash::vk;
use winit::event_loop::EventLoop;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct VulkanContext {
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    device: Device,
}

impl VulkanContext {
    pub(crate) fn new<A: EngineCallbackHandler>(
        event_loop: &EventLoop<()>,
        app: &mut A,
    ) -> anyhow::Result<Self> {
        let instance = Instance::new(event_loop, app)?;

        app.on_instance(&instance);

        let physical_device = instance.select_physical_device(app)?;
        app.on_physical_device(physical_device, &instance);

        let device = Device::new(event_loop, &instance, physical_device, app)?;
        app.on_device(&device);

        Ok(Self {
            instance,
            physical_device,
            device,
        })
    }

    pub(crate) fn create_surface<T: HasWindowHandle + HasDisplayHandle>(
        &self,
        window: T,
    ) -> Result<vk::SurfaceKHR, CreateSurfaceError> {
        let window_handle = window.window_handle()?;
        let display_handle = window.display_handle()?;

        Ok(unsafe {
            ash_window::create_surface(
                self.instance.entry(),
                self.instance.instance(),
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )?
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn create_semaphore(&self) -> VkResult<vk::Semaphore> {
        unsafe {
            self.device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
    }

    pub fn create_fence(&self) -> VkResult<vk::Fence> {
        unsafe {
            self.device
                .create_fence(&vk::FenceCreateInfo::default(), None)
        }
    }

    pub fn create_fence_signaled(&self) -> VkResult<vk::Fence> {
        unsafe {
            self.device.create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
        }
    }

    pub fn query_present_modes(
        &self,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            self.instance
                .loader()
                .surface()
                .get_physical_device_surface_present_modes(self.physical_device, surface)
        }
    }

    pub fn query_surface_formats(
        &self,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.instance
                .loader()
                .surface()
                .get_physical_device_surface_formats(self.physical_device, surface)
        }
    }

    pub fn query_surface_capabilities(
        &self,
        surface: vk::SurfaceKHR,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.instance
                .loader()
                .surface()
                .get_physical_device_surface_capabilities(self.physical_device, surface)
        }
    }

    pub fn create_semaphores(&self) -> VkResult<FrameSet<vk::Semaphore>> {
        FrameSet::<VkResult<vk::Semaphore>>::create_factory(|_| self.create_semaphore()).promote_errors()
    }

    pub fn create_fences(&self) -> VkResult<FrameSet<vk::Fence>> {
        FrameSet::<VkResult<vk::Fence>>::create_factory(|_| self.create_fence()).promote_errors()
    }

    pub fn create_fences_signaled(&self) -> VkResult<FrameSet<vk::Fence>> {
        FrameSet::<VkResult<vk::Fence>>::create_factory(|_| self.create_fence_signaled()).promote_errors()
    }

    pub fn wait_for_fence(&self, fence: vk::Fence) -> VkResult<()> {
        unsafe { self.device.wait_for_fences(&[fence], true, u64::MAX) }
    }

    pub fn wait_for_fences(&self, fences: &[vk::Fence]) -> VkResult<()> {
        unsafe { self.device.wait_for_fences(fences, true, u64::MAX) }
    }

    pub fn reset_fence(&self, fence: vk::Fence) -> VkResult<()> {
        unsafe { self.device.reset_fences(&[fence]) }
    }

    pub fn reset_fences(&self, fences: &[vk::Fence]) -> VkResult<()> {
        unsafe { self.device.reset_fences(fences) }
    }
}
