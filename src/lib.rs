#![allow(missing_docs)]
pub extern crate ash;
extern crate core;
pub extern crate winit;

use std::cell::RefCell;
use crate::errors::CreateWindowError;
use crate::render::context::device::Device;
use crate::render::context::instance::Instance;
use crate::render::context::VulkanContext;
use app::feature_request::{
    DeviceFeatureRequest, ExtensionRequest, FeatureStructs, QueueRequest,
};
use ash::vk;
use render::window::WindowData;
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::sync;
use std::sync::Arc;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowAttributes, WindowId};

pub mod app;
pub mod errors;
pub mod render;
pub mod utils;



pub const ENGINE_NAME: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"neuron-engine\0") };
pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 1, 0); // TODO: use env! for version



pub struct Engine {
    windows: HashMap<WindowId, Arc<RefCell<WindowData>>>,
    vulkan_context: Arc<VulkanContext>,
}

#[allow(unused_variables)]
pub trait EngineCallbackHandler {
    fn name(&self) -> &str {
        ""
    }
    fn version(&self) -> (u32, u32, u32) {
        (1, 0, 0)
    }

    fn on_request_device_extensions(&mut self, requested_extensions: &mut Vec<ExtensionRequest>) {}
    fn on_request_instance_extensions(&mut self, requested_extensions: &mut Vec<ExtensionRequest>) {
    }

    fn on_resolve_device_extensions(&mut self, extensions: &HashSet<&'static CStr>) {}
    fn on_resolve_instance_extensions(&mut self, extensions: &HashSet<&'static CStr>) {}

    fn on_request_features(&mut self, requested_features: &mut Vec<DeviceFeatureRequest>) {}
    fn on_resolve_features<'a>(&mut self, features: &FeatureStructs<'a>) {}

    ///
    /// This function is not self-mutable since there is no cross-system guarantees on this (unlike the extensions functions which will always be called once at the same point in execution on all systems).
    /// TODO: wrap physical devices with an easier to work with wrapper.
    fn validate_physical_device(
        &self,
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
    ) -> bool {
        true
    }

    fn on_instance(&mut self, instance: &Instance) {}
    fn on_physical_device(
        &mut self,
        physical_device: vk::PhysicalDevice,
        instance: &Instance,
    ) {
    }

    fn on_device(&mut self, device: &Device) {}

    /// Use this function to request queues to be obtained (use only if absolutely required, most of the time the system allocates the right queues. The main use case is if you need video queues).
    ///
    /// # Arguments
    ///
    /// * `existing_requests`: The queues being requested by the system already
    /// * `families`: The family properties (suggest to use the enumeration iterator of a `Vec` to access so you have the index).
    ///
    /// returns: `Vec<QueueRequest>` containing a set of requests for queues (will be flatted with the existing requests after this is called).
    ///
    /// # Examples
    ///
    /// ```
    /// use neuron_engine::{EngineCallbackHandler, app::feature_request::QueueRequest};
    /// use neuron_engine::render::context::queues::QueueLabel;
    /// use ash::vk::{QueueFamilyProperties, QueueFlags};
    ///
    /// struct MyHandler;
    ///
    /// impl EngineCallbackHandler for MyHandler {
    ///     fn on_queue_selection(&mut self, existing_requests: &[QueueRequest], families: Vec<QueueFamilyProperties>) -> Vec<QueueRequest> {
    ///         let mut video_encode_queue: Option<usize> = None;
    ///         let mut video_decode_queue: Option<usize> = None;
    ///
    ///         families.iter().enumerate().for_each(|(index, properties)| {
    ///             if video_encode_queue.is_none() && properties.queue_flags.contains(QueueFlags::VIDEO_ENCODE_KHR) {
    ///                 video_encode_queue = Some(index);
    ///             }
    ///
    ///             if video_decode_queue.is_none() && properties.queue_flags.contains(QueueFlags::VIDEO_DECODE_KHR) {
    ///                 video_decode_queue = Some(index);
    ///             }
    ///         });
    ///
    ///         let mut requests: Vec<QueueRequest> = Vec::new();
    ///         if let Some(i) = video_encode_queue {
    ///             requests.push(QueueRequest { family: i as u32, count: 1, label: Some(QueueLabel::VideoEncode), allow_merge: true });
    ///         }
    ///
    ///         if let Some(i) = video_decode_queue {
    ///             requests.push(QueueRequest { family: i as u32, count: 1, label: Some(QueueLabel::VideoDecode), allow_merge: true });
    ///         }
    ///
    ///         requests
    ///     }
    /// }

    ///
    ///
    /// ```
    fn on_queue_selection(
        &mut self,
        existing_requests: &[QueueRequest],
        families: Vec<vk::QueueFamilyProperties>,
    ) -> anyhow::Result<Vec<QueueRequest>> {
        Ok(vec![])
    }

    fn on_engine_ready(&mut self, engine: &mut Engine) -> anyhow::Result<()> { Ok(()) }
}

impl Engine {
    pub(crate) fn init<A: EngineCallbackHandler>(
        event_loop: &EventLoop<()>,
        app: &mut A,
    ) -> anyhow::Result<Self> {
        let mut engine = Self {
            windows: HashMap::new(),
            vulkan_context: Arc::new(VulkanContext::new(event_loop, app)?),
        };

        app.on_engine_ready(&mut engine)?;

        Ok(engine)
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<sync::Weak<RefCell<WindowData>>, CreateWindowError> {
        let window = Arc::new(RefCell::new(WindowData::new(
            self,
            event_loop.create_window(attributes)?,
        )?));
        let window_id = window.borrow().window().id();
        let weakref = Arc::downgrade(&window);
        self.windows.insert(window_id, window);
        Ok(weakref)
    }

    pub fn close_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn windows(&self) -> &HashMap<WindowId, Arc<RefCell<WindowData>>> {
        &self.windows
    }

    pub fn vulkan(&self) -> Arc<VulkanContext> {
        self.vulkan_context.clone()
    }

    pub fn get_window(&self, window_id: &WindowId) -> Option<&Arc<RefCell<WindowData>>> {
        self.windows.get(window_id)
    }
}
