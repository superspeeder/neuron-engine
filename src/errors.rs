use ash::vk;
use thiserror::Error;
pub use winit::error::OsError;
pub use winit::raw_window_handle::HandleError;

#[derive(Debug, Error)]
pub enum CreateSurfaceError {
    #[error(transparent)]
    VulkanError(#[from] vk::Result),

    #[error(transparent)]
    HandleError(#[from] HandleError),
}

#[derive(Debug, Error)]
pub enum CreateWindowError {
    #[error(transparent)]
    OsError(#[from] OsError),

    #[error(transparent)]
    CreateSurfaceError(#[from] CreateSurfaceError),

    #[error(transparent)]
    VulkanError(#[from] vk::Result),
}

#[derive(Debug, Error)]
pub enum QueueRequestValidationError {
    #[error("Not enough queues in queue family {family:?} (requested {req:?}, available {avail:?})")]
    NotEnoughQueuesInFamily {
        family: u32,
        req: u32,
        avail: u32,
    }
}