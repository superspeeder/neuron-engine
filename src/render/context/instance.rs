use crate::app::feature_request::ExtensionRequest;
use crate::{ENGINE_NAME, ENGINE_VERSION, EngineCallbackHandler};
use anyhow::anyhow;
use ash::{khr, vk};
use log::{debug, info, trace};
use std::collections::HashSet;
use std::ffi::{CStr, CString, c_char};
use std::ops::{Deref, DerefMut};
use winit::event_loop::EventLoop;
use winit::raw_window_handle::HasDisplayHandle;

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
    loader: InstanceLoader,
}

impl Instance {
    pub fn new<A: EngineCallbackHandler>(
        event_loop: &EventLoop<()>,
        app: &mut A,
    ) -> anyhow::Result<Self> {
        #[cfg(feature = "vulkan_linked")]
        let entry = unsafe {
            info!("[vulkan/setup] Using linked vulkan entry point");
            ash::Entry::linked()
        };
        #[cfg(not(feature = "vulkan_linked"))]
        let entry = unsafe {
            info!("[vulkan/setup] Loading vulkan entry point dynamically");
            ash::Entry::load()
        }?;

        let required_extensions =
            ash_window::enumerate_required_extensions(event_loop.display_handle()?.as_raw())?;

        let mut requested_extensions = required_extensions
            .iter()
            .cloned()
            .map(|p| ExtensionRequest::required(unsafe { CStr::from_ptr(p) }))
            .collect::<Vec<ExtensionRequest>>();

        debug!(
            "[instance/extensions] System required instance extensions: {}",
            required_extensions
                .iter()
                .map(|e| unsafe { CStr::from_ptr(e.clone()) }
                    .to_str()
                    .unwrap()
                    .to_owned())
                .reduce(|a, b| format!("{}, {}", a, b))
                .unwrap_or("".to_owned())
        );

        app.on_request_instance_extensions(&mut requested_extensions);

        trace!("[instance/extensions] Requested instance extensions");
        requested_extensions
            .iter()
            .for_each(|ext| trace!("[instance/extensions/#] - {:?}", ext));

        let extension_properties = unsafe { entry.enumerate_instance_extension_properties(None) }?;

        let available_extensions = extension_properties
            .iter()
            .map(|props| unsafe { CStr::from_ptr(props.extension_name.as_ptr()).to_owned() })
            .collect::<HashSet<CString>>();

        trace!("[instance/extensions] Available instance extensions:");
        available_extensions
            .iter()
            .for_each(|ext| trace!("[instance/extensions/#] - {:?}", ext));

        let missing = requested_extensions
            .iter()
            .filter(|req| req.required && !available_extensions.contains(&req.name.to_owned()))
            .map(|req| req.name)
            .collect::<Vec<&'static CStr>>();

        if !missing.is_empty() {
            return Err(anyhow!(
                "Missing required instance extensions: {:?}",
                missing
            ));
        }

        let missing_optionals = requested_extensions
            .iter()
            .filter(|req| !req.required && !available_extensions.contains(&req.name.to_owned()))
            .map(|req| req.name)
            .collect::<Vec<&'static CStr>>();

        if !missing_optionals.is_empty() {
            debug!("[instance/extensions] Missing optional instance extensions:");
            missing_optionals
                .iter()
                .for_each(|e| debug!("[instance/extensions/#] - {:?}", e));
        }

        let extensions_set = requested_extensions
            .iter()
            .filter(|req| available_extensions.contains(&req.name.to_owned()))
            .map(|req| req.name)
            .collect::<HashSet<&'static CStr>>();

        debug!("[instance/extensions] Resolved instance extensions:");
        extensions_set
            .iter()
            .for_each(|e| debug!("[instance/extensions/#] - {:?}", e));

        app.on_resolve_instance_extensions(&extensions_set);

        let extensions = extensions_set
            .iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<*const c_char>>();

        let app_name = CString::new(app.name())?;
        let app_version = app.version();

        debug!("[instance/configuration] Application name: {:?}", app_name);
        debug!(
            "[instance/configuration] Application version: {:?}",
            app_version
        );

        let application_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3)
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION)
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(
                0,
                app_version.0,
                app_version.1,
                app_version.2,
            ));

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_extension_names(&extensions)
            .application_info(&application_info);

        let instance = unsafe { entry.create_instance(&create_info, None) }?;

        info!("[vulkan/instance] Successfully created vulkan instance.");

        let loader = InstanceLoader::load(&entry, &instance);

        Ok(Instance {
            entry,
            instance,
            loader,
        })
    }

    pub fn select_physical_device<A: EngineCallbackHandler>(
        &self,
        app: &mut A,
    ) -> anyhow::Result<vk::PhysicalDevice> {
        let physical_devices = unsafe { self.enumerate_physical_devices() }?;

        for physical_device in physical_devices {
            if app.validate_physical_device(physical_device, &self.instance) {
                let properties = unsafe { self.get_physical_device_properties(physical_device) };
                info!(
                    "[vulkan/physical device] Selected Physical Device: {}",
                    properties.device_name_as_c_str()?.to_str()?
                );
                return Ok(physical_device);
            }
        }

        Err(anyhow!("Failed to find a suitable physical device"))
    }

    pub fn load_extension<E, F: FnOnce(&ash::Entry, &ash::Instance) -> E>(&self, f: F) -> E {
        f(&self.entry, &self.instance)
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn loader(&self) -> &InstanceLoader {
        &self.loader
    }
}

impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}

pub struct InstanceLoader {
    surface: khr::surface::Instance,
}

impl InstanceLoader {
    pub fn load(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        Self {
            surface: khr::surface::Instance::new(entry, instance),
        }
    }

    pub fn surface(&self) -> &khr::surface::Instance {
        &self.surface
    }
}
