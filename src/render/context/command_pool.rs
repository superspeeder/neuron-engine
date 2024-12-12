use std::sync::Arc;
use ash::prelude::VkResult;
use ash::vk;
use crate::render::context::VulkanContext;
use crate::render::frame_set::{FrameSet, MAX_FRAMES_IN_FLIGHT};

pub struct CommandPool {
    vulkan_context: Arc<VulkanContext>,
    pool: vk::CommandPool,
    queue_family: u32,
}

impl CommandPool {
    pub fn new(vulkan_context: Arc<VulkanContext>, queue_family: u32, allow_reset: bool) -> VkResult<Self> {
        let pool = unsafe {
            vulkan_context.device().create_command_pool(&vk::CommandPoolCreateInfo::default()
                .flags(if allow_reset { vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER } else { vk::CommandPoolCreateFlags::empty() })
                .queue_family_index(queue_family), None)
        }?;

        Ok(Self {
            vulkan_context,
            pool,
            queue_family,
        })
    }

    pub fn allocate_command_buffer_set(&self) -> VkResult<FrameSet<vk::CommandBuffer>> {
        let command_buffers = self.allocate_command_buffers(2)?;
        Ok(FrameSet::from([command_buffers[0], command_buffers[1]]))
    }

    pub fn allocate_command_buffers(&self, count: usize) -> VkResult<Vec<vk::CommandBuffer>> {
        unsafe {
            self.vulkan_context.device().allocate_command_buffers(&vk::CommandBufferAllocateInfo::default()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(count as u32)
                .command_pool(self.pool))
        }
    }
}