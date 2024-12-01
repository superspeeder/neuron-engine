use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;
use winit::window::{Window, WindowId};

pub(crate) struct WindowData<'a> {
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'a>,
    window: Window,
}

pub struct App<'a> {
    wgpu_instance: Arc<wgpu::Instance>,
    window: Option<Arc<WindowData<'a>>>
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            wgpu_instance: Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::VULKAN,
                flags: wgpu::InstanceFlags::debugging(),
                ..Default::default()
            })),
            window: None,
        }
    }

    pub fn window(&self) -> Option<Arc<WindowData>> {
        self.window.clone()
    }
}

#[derive(Error, Debug)]
pub enum CreateWindowDataError {
    #[error(transparent)]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("Valid adapter not found")]
    AdapterNotFound,
}

impl WindowData<'_> {
    pub async fn new(instance: &wgpu::Instance, window: Window) -> Result<Self, CreateWindowDataError> {
        let surface = unsafe { instance.create_surface(&window) }?;

        let adapter= instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await.map_or(Err(CreateWindowDataError::AdapterNotFound), |a| Ok(a))?;

        Ok(Self {
            adapter,
            surface,
            window,
        })
    }
}