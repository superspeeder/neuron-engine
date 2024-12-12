use crate::render::context::device::LazyQueue;
use crate::render::context::VulkanContext;
use ash::prelude::VkResult;
use ash::vk;
use neuron_procmacro::sealed;
use std::sync::Arc;

pub struct CommandRecorder<'a> {
    command_buffer: &'a vk::CommandBuffer,
    vulkan: Arc<VulkanContext>,
    auto_submit: Option<AutoSubmitInfo>,
}

pub enum GenericSemaphore {
    Binary(vk::Semaphore, vk::PipelineStageFlags2),
    Timeline(vk::Semaphore, u64, vk::PipelineStageFlags2),
}

pub struct SemaphoreInfo {
    pub semaphore: GenericSemaphore,
    pub device_index: Option<u32>,
}

pub struct CommandBufferSyncInfo {
    wait_semaphores: Vec<SemaphoreInfo>,
    signal_semaphores: Vec<SemaphoreInfo>,
    command_buffers: Vec<vk::CommandBuffer>,
    fence: Option<vk::Fence>,
}

pub struct AutoSubmitInfo {
    queue: LazyQueue,
    sync_info: CommandBufferSyncInfo,
}

impl<'a> CommandRecorder<'a> {
    pub(crate) fn wrapper(
        command_buffer: &'a vk::CommandBuffer,
        vulkan: Arc<VulkanContext>,
    ) -> Self {
        Self {
            command_buffer,
            vulkan,
            auto_submit: None,
        }
    }

    pub(crate) fn wrapper_auto_submit(
        command_buffer: &'a vk::CommandBuffer,
        vulkan: Arc<VulkanContext>,
        auto_submit: AutoSubmitInfo,
    ) -> Self {
        Self {
            command_buffer,
            vulkan,
            auto_submit: Some(auto_submit),
        }
    }
}

extend_type!(vk::CommandBuffer) {

}

#[sealed(vk::CommandBuffer)]
pub trait CommandBufferExt {
    fn begin(&self, vulkan: Arc<VulkanContext>, one_time_submit: bool)
    -> VkResult<CommandRecorder>;
    fn begin_auto_submit(
        &self,
        vulkan: Arc<VulkanContext>,
        one_time_submit: bool,
        auto_submit_info: AutoSubmitInfo,
    ) -> VkResult<CommandRecorder>;
}

impl<'a> CommandBufferExt for vk::CommandBuffer {
    fn begin(
        &self,
        vulkan: Arc<VulkanContext>,
        one_time_submit: bool,
    ) -> VkResult<CommandRecorder> {
        unsafe {
            vulkan.device().begin_command_buffer(
                self.clone(),
                &vk::CommandBufferBeginInfo::default().flags(if one_time_submit {
                    vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT
                } else {
                    vk::CommandBufferUsageFlags::empty()
                }),
            )?;
        }

        Ok(CommandRecorder::wrapper(self, vulkan))
    }

    fn begin_auto_submit(
        &self,
        vulkan: Arc<VulkanContext>,
        one_time_submit: bool,
        auto_submit_info: AutoSubmitInfo,
    ) -> VkResult<CommandRecorder> {
        unsafe {
            vulkan.device().begin_command_buffer(
                self.clone(),
                &vk::CommandBufferBeginInfo::default().flags(if one_time_submit {
                    vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT
                } else {
                    vk::CommandBufferUsageFlags::empty()
                }),
            )?;
        }

        Ok(CommandRecorder::wrapper_auto_submit(
            self,
            vulkan,
            auto_submit_info,
        ))
    }
}

impl<'a> Drop for CommandRecorder<'a> {
    fn drop(&mut self) {
        unsafe {
            if let Ok(_) = self.vulkan.device().end_command_buffer(self.command_buffer.clone()) {
                if let Some(auto_submit) = &self.auto_submit {
                    _ = auto_submit.submit(&[self.command_buffer.clone()], self.vulkan.clone());
                }
            }
        }
    }
}

impl AutoSubmitInfo {
    pub(crate) fn submit(&self, command_buffers: &[vk::CommandBuffer], vulkan: Arc<VulkanContext>) -> VkResult<()> {

    }
}