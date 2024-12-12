use crate::EngineCallbackHandler;
use crate::app::feature_request::{
    DeviceFeature, DeviceFeatureRequest, ExtensionRequest, FeatureStructs, QueueRequest,
};
use crate::errors::QueueRequestValidationError;
use crate::render::context::instance::Instance;
use crate::render::context::platform;
use crate::render::context::queues::{QueueLabel, QueueLabels, QueueRef, UnlabeledQueues};
use anyhow::anyhow;
use ash::{khr, vk};
use log::{debug, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString, c_char};
use std::iter::repeat_n;
use std::ops::{Deref, DerefMut};
use ash::prelude::VkResult;
use winit::event_loop::EventLoop;
use winit::raw_window_handle::HasDisplayHandle;

const REQUIRED_DEVICE_EXTENSIONS: &'static [ExtensionRequest] =
    &[ExtensionRequest::required(khr::swapchain::NAME)];

const REQUIRED_FEATURES: &'static [DeviceFeatureRequest] = &[
    DeviceFeatureRequest::required(DeviceFeature::DynamicRendering),
    DeviceFeatureRequest::required(DeviceFeature::GeometryShader),
    DeviceFeatureRequest::required(DeviceFeature::TessellationShader),
    DeviceFeatureRequest::required(DeviceFeature::WideLines),
    DeviceFeatureRequest::required(DeviceFeature::LargePoints),
    DeviceFeatureRequest::required(DeviceFeature::Synchronization2),
    DeviceFeatureRequest::required(DeviceFeature::TimelineSemaphore),
];

pub struct Device {
    device: ash::Device,
    queues: HashMap<u32, Vec<vk::Queue>>,
    queue_labels: QueueLabels,
    unlabeled_queues: UnlabeledQueues,
    loader: DeviceLoader,
}

impl Device {
    pub fn new<A: EngineCallbackHandler>(
        event_loop: &EventLoop<()>,
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        app: &mut A,
    ) -> anyhow::Result<Device> {
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics: Option<u32> = None;
        let mut transfer: Option<u32> = None;
        let mut compute: Option<u32> = None;
        let mut presentation: Option<u32> = None;
        let mut queue_availability: HashMap<u32, u32> = HashMap::new();
        let mut total_queue_availability: HashMap<u32, u32> = HashMap::new();

        let raw_display_handle = event_loop.display_handle()?.as_raw();

        queue_family_properties
            .iter()
            .enumerate()
            .for_each(|(i, props)| {
                queue_availability.insert(i as u32, props.queue_count);
                total_queue_availability.insert(i as u32, props.queue_count);
                if props.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
                    graphics = Some(i as u32);
                    trace!("[device/queues] Found graphics queue: {:?}", i);
                }

                if props.queue_flags.contains(vk::QueueFlags::TRANSFER)
                    && !props
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
                    && transfer.is_none()
                {
                    transfer = Some(i as u32);
                    trace!("[device/queues] Found exclusive transfer queue: {:?}", i);
                }

                if props.queue_flags.contains(vk::QueueFlags::COMPUTE) && compute.is_none() {
                    compute = Some(i as u32);
                    trace!("[device/queues] Found compute queue: {:?}", i);
                }

                if presentation.is_none()
                    && platform::can_present(
                        &raw_display_handle,
                        i as u32,
                        instance,
                        physical_device,
                    )
                {
                    presentation = Some(i as u32);
                    trace!("[device/queues] Found presentation queue: {:?}", i);
                }
            });

        let Some(graphics) = graphics else {
            return Err(anyhow!("No graphics queue family found"));
        };

        let Some(compute) = compute else {
            return Err(anyhow!(
                "No compute queue family found (this indicates a non-conformant vulkan implementation)"
            ));
        };

        if transfer.is_none() {
            debug!(
                "[device/queues] No exclusive transfer queue found, falling back on primary graphics queue."
            );
        }

        let mut queue_requests = vec![
            // QueueRequest {
            //     family: graphics,
            //     count: 1,
            //     label: Some(QueueLabel::Graphics),
            //     allow_merge: true,
            // },
            // QueueRequest {
            //     family: transfer.unwrap_or(graphics),
            //     count: 1,
            //     label: Some(QueueLabel::Transfer),
            //     allow_merge: false,
            // },
            // QueueRequest {
            //     family: compute,
            //     count: 1,
            //     label: Some(QueueLabel::Compute),
            //     allow_merge: true,
            // },
            QueueRequest::flexible_labeled(graphics, 1, QueueLabel::Graphics),
            QueueRequest::flexible_labeled(compute, 1, QueueLabel::Compute),
        ];

        if let Some(transfer) = transfer {
            queue_requests.push(QueueRequest::strict_labeled(
                transfer,
                1,
                QueueLabel::Transfer,
            ));
        } else {
            warn!("No exclusive transfer queue family. Falling back on primary graphics queue.");
            queue_requests.push(QueueRequest::flexible_labeled(
                graphics,
                1,
                QueueLabel::Transfer,
            ));
        }

        if let Some(presentation) = presentation {
            queue_requests.push(QueueRequest::flexible_labeled(
                presentation,
                1,
                QueueLabel::Presentation,
            ))
        } else {
            warn!(
                "No queue family supports presentation. Operations running on windows will not work properly."
            );
        };

        {
            let mut user_requests =
                app.on_queue_selection(queue_requests.as_slice(), queue_family_properties)?;
            queue_requests.append(&mut user_requests);
        }

        // Validate requests now
        let mut strict_requests: HashMap<u32, u32> = HashMap::new(); // all of these must be exclusives
        let mut flexible_requests: HashMap<u32, u32> = HashMap::new(); // all of these may not be exclusives (allowed to merge together)

        let mut strict_labels: HashMap<u32, Vec<QueueLabel>> = HashMap::new();
        let mut flexible_labels: HashMap<u32, Vec<QueueLabel>> = HashMap::new();

        let mut strict_labels_counts: HashMap<QueueLabel, HashMap<u32, usize>> = HashMap::new();
        let mut flexible_labels_counts: HashMap<QueueLabel, HashMap<u32, usize>> = HashMap::new();

        trace!("[device/queues] Processing and validating queue requests");
        for req in queue_requests {
            if req.allow_merge {
                if let Some(count) = flexible_requests.get(&(req.family as u32)).cloned() {
                    flexible_requests.insert(req.family as u32, count + req.count);
                    trace!(
                        "[device/queues/flexible request] (update) family: {:?}, count: {:?} (old: {:?})",
                        req.family,
                        count + req.count,
                        count
                    );
                } else {
                    flexible_requests.insert(req.family as u32, req.count);
                    trace!(
                        "[device/queues/flexible request] family: {:?}, count: {:?}",
                        req.family, req.count
                    );
                }

                if let Some(label) = req.label {
                    trace!(
                        "[device/queues/flexible request] label: {:?}, family: {:?}, count: {:?}",
                        label, req.family, req.count
                    );

                    if let Some(labels) = flexible_labels.get_mut(&req.family) {
                        labels.push(label);
                    } else {
                        flexible_labels.insert(req.family, vec![label]);
                    }

                    if let Some(counts) = flexible_labels_counts.get_mut(&label) {
                        if let Some(count) = counts.get_mut(&req.family) {
                            *count += req.count as usize;
                        } else {
                            counts.insert(req.family, req.count as usize);
                        }
                    } else {
                        flexible_labels_counts
                            .insert(label, HashMap::from([(req.family, req.count as usize)]));
                    }
                }
            } else {
                if let Some(count) = strict_requests.get(&(req.family)).cloned() {
                    strict_requests.insert(req.family, count + req.count);
                    trace!(
                        "[device/queues/strict request] (update) family: {:?}, count: {:?} (old: {:?})",
                        req.family,
                        count + req.count,
                        count
                    );
                } else {
                    strict_requests.insert(req.family, req.count);
                    trace!(
                        "[device/queues/strict request] family: {:?}, count: {:?}",
                        req.family, req.count
                    );
                }

                if let Some(label) = req.label {
                    trace!(
                        "[device/queues/strict request] label: {:?}, family: {:?}",
                        label, req.family
                    );

                    if let Some(labels) = strict_labels.get_mut(&req.family) {
                        labels.push(label);
                    } else {
                        strict_labels.insert(req.family, vec![label]);
                    }

                    if let Some(counts) = strict_labels_counts.get_mut(&label) {
                        if let Some(count) = counts.get_mut(&req.family) {
                            *count += req.count as usize;
                        } else {
                            counts.insert(req.family, req.count as usize);
                        }
                    } else {
                        strict_labels_counts
                            .insert(label, HashMap::from([(req.family, req.count as usize)]));
                    }
                }
            }
        }

        let mut unlabeled = UnlabeledQueues::new();
        let mut labeled = QueueLabels::new();

        let mut flexible_starts: HashMap<u32, u32> = HashMap::new();

        for (family, mut count) in strict_requests.clone() {
            let mut end_index: u32 = 0;
            trace!(
                "[device/queues/strict request/processing] Processing request: (family: {:?}, count: {:?})",
                family, count
            );

            if let Some(available) = queue_availability.get_mut(&family) {
                if count > available.clone() {
                    return Err(QueueRequestValidationError::NotEnoughQueuesInFamily {
                        family,
                        req: count + flexible_requests.get(&family).map(|_| 1).unwrap_or(0),
                        avail: total_queue_availability.get(&family).cloned().unwrap_or(0),
                    }
                    .into());
                }

                trace!(
                    "[device/queues/strict request/processing] Allocating {:?} queues from queue family {:?} (out of {:?} total available)",
                    count, family, available
                );

                *available -= count;
            }

            if let Some(labels) = strict_labels.get(&family) {
                trace!("[device/queues/strict request] Beginning label allocation");
                for label in labels {
                    let rc = strict_labels_counts
                        .get(&label)
                        .and_then(|counts| counts.get(&family))
                        .cloned()
                        .unwrap_or(1);
                    for _ in 0..rc {
                        trace!(
                            "[device/queues/strict request/label allocation] Allocating queue #{:?} in family {:?} to label {:?}",
                            end_index, family, label
                        );
                        if let Some(queues) = labeled.get_mut(label) {
                            queues.push(QueueRef {
                                family,
                                index: end_index,
                            });
                        } else {
                            labeled.insert(label.clone(), vec![QueueRef {
                                family,
                                index: end_index,
                            }]);
                        }
                        end_index += 1;
                        count -= 1;
                    }
                }
            }

            // unlabeled
            if count > 0 {
                trace!(
                    "[device/queues/strict request/processing] Marked {:?} queues (#{:?} through #{:?}) in family {:?} as unlabeled",
                    count,
                    end_index,
                    end_index + count - 1,
                    family
                );
                unlabeled.insert(
                    family,
                    (end_index..end_index + count).collect::<HashSet<u32>>(),
                );
                end_index += count;
            }

            flexible_starts.insert(family, end_index);
            trace!(
                "[device/queues/strict request/processing] Flexible requests on family {:?} will start from queue #{:?}",
                family, end_index
            );
        }

        for (family, mut count) in flexible_requests {
            trace!(
                "[device/queues/flexible request/processing] Processing request: (family: {:?}, count: {:?})",
                family, count
            );

            if let Some(total) = total_queue_availability.get(&family).cloned() {
                if let Some(available) = queue_availability.get_mut(&family) {
                    trace!(
                        "[device/queues/flexible request/processing] {:?} out of {:?} queues available in family {:?}",
                        available, total, family
                    );
                    if available.clone() <= 0 {
                        return Err(QueueRequestValidationError::NotEnoughQueuesInFamily {
                            family: family.clone(),
                            req: strict_requests.get(&family).cloned().unwrap_or(0) + 1,
                            avail: total,
                        }
                        .into());
                    }

                    if count > available.clone() {
                        trace!(
                            "[device/queues/flexible request/processing] More queues requested than available queues for family {:?}, some will be merged. (requested {:?}, available {:?})",
                            family, count, available
                        );
                        *available = 0;
                    } else {
                        trace!(
                            "[device/queues/flexible request/processing] No queue merging is required for family {:?} (requested {:?}, available {:?})",
                            family, count, available
                        );
                        *available -= count;
                    }

                    let flexible_range =
                        flexible_starts.get(&family).cloned().unwrap_or(0)..total;
                    let mut o_index = 0;

                    trace!(
                        "[device/queues/flexible request/processing] Flexible queue range is queues #{:?} through #{:?} for family {:?}",
                        flexible_range.start,
                        flexible_range.end - 1,
                        family
                    );

                    if let Some(labels) = flexible_labels.get(&family) {
                        for label in labels {
                            let rc = flexible_labels_counts
                                .get(&label)
                                .and_then(|counts| counts.get(&family))
                                .cloned()
                                .unwrap_or(1);
                            trace!(
                                "[device/queues/flexible request/label allocation] Will allocate {:?} queues in family {:?} to label {:?}",
                                rc, family, label
                            );
                            for _ in 0..rc {
                                let index =
                                    flexible_range.start + (o_index % flexible_range.len()) as u32;
                                if let Some(queues) = labeled.get_mut(label) {
                                    queues.push(QueueRef { family, index });
                                } else {
                                    labeled.insert(label.clone(), vec![QueueRef { family, index }]);
                                }

                                trace!(
                                    "[device/queues/flexible request/label allocation] Allocating queue #{:?} in family {:?} to label {:?}",
                                    index, family, label
                                );

                                o_index += 1;
                                count -= 1;
                            }
                        }
                    }

                    // unlabeled
                    if count > 0 {
                        let indices = (o_index..o_index + count as usize)
                            .map(|i| flexible_range.start + (i % flexible_range.len()) as u32)
                            .collect::<HashSet<u32>>();

                        trace!(
                            "[device/queues/flexible request/processing] Marked {:?} queues in family {:?} as unlabeled (in virtual space, range is: {:?} through {:?}, maps to indices: {:?})",
                            count,
                            family,
                            o_index,
                            o_index + (count as usize) - 1,
                            indices
                        );

                        unlabeled.insert(family, indices);
                    }
                }
            } else {
                return Err(QueueRequestValidationError::NotEnoughQueuesInFamily {
                    family,
                    req: strict_requests.get(&family).cloned().unwrap_or(0) + 1,
                    avail: 0,
                }
                .into());
            }
        }

        let mut device_queue_create_infos = Vec::<vk::DeviceQueueCreateInfo>::new();
        let mut priorities: HashMap<u32, Vec<f32>> = HashMap::new();

        for (f, total) in total_queue_availability {
            if let Some(real) = queue_availability.get(&f) {
                if real.clone() == total {
                    trace!(
                        "[device/queues/configure] Skipping queue family {:?} (no requests)",
                        f
                    );
                    continue;
                }

                let this_priorities = repeat_n(1.0f32, (total - real) as usize).collect();
                trace!(
                    "[device/queues/configure] Priorities for {:?} queues allocated in family {:?}: {:?}",
                    total - real,
                    f,
                    this_priorities
                );
                priorities.insert(f, this_priorities);
            }
        }

        for (f, prio) in priorities.iter() {
            device_queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_priorities(prio.as_slice())
                    .queue_family_index(f.clone()),
            );
            trace!(
                "[device/queues/configure] Queue family {:?} configured for {:?} queues",
                f,
                prio.len()
            );
        }

        let mut requested_extensions: Vec<ExtensionRequest> = Vec::from(REQUIRED_DEVICE_EXTENSIONS);
        trace!("[device/extensions] Beginning device extension selection");
        trace!("[device/extensions] Engine requests:");
        requested_extensions
            .iter()
            .for_each(|e| trace!("[device/extensions/#] - {:?}", e));

        let extension_properties =
            unsafe { instance.enumerate_device_extension_properties(physical_device) }?;

        let available_extensions = extension_properties
            .iter()
            .map(|props| unsafe { CStr::from_ptr(props.extension_name.as_ptr()).to_owned() })
            .collect::<HashSet<CString>>();

        trace!("[device/extensions] Available Extensions");
        available_extensions
            .iter()
            .for_each(|e| trace!("[device/extensions/#] - {:?}", e));

        app.on_request_device_extensions(&mut requested_extensions);

        trace!("[device/extensions] Extension Requests:");
        requested_extensions
            .iter()
            .for_each(|e| trace!("[device/extensions/#] - {:?}", e));

        let missing = requested_extensions
            .iter()
            .filter(|req| req.required && !available_extensions.contains(&req.name.to_owned()))
            .map(|req| req.name)
            .collect::<Vec<&'static CStr>>();

        let missing_optionals = requested_extensions
            .iter()
            .filter(|req| !req.required && !available_extensions.contains(&req.name.to_owned()))
            .map(|req| req.name)
            .collect::<Vec<&'static CStr>>();

        if !missing.is_empty() {
            return Err(anyhow!("Missing required extensions: {:?}", missing));
        }

        if !missing_optionals.is_empty() {
            debug!("[device/extensions] Missing optional device extensions:");
            missing_optionals
                .iter()
                .for_each(|e| debug!("[device/extensions/#] - {:?}", e));
        }

        let extensions_set = requested_extensions
            .iter()
            .filter(|req| available_extensions.contains(&req.name.to_owned()))
            .map(|req| req.name)
            .collect::<HashSet<&'static CStr>>();

        debug!("[device/extensions] Resolved extensions:");
        extensions_set
            .iter()
            .for_each(|e| debug!("[device/extensions/#] - {:?}", e));

        app.on_resolve_device_extensions(&extensions_set);

        let extensions = extensions_set
            .iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut requested_features: Vec<DeviceFeatureRequest> = Vec::from(REQUIRED_FEATURES);
        debug!("[device/features] Engine requested features:");
        requested_features
            .iter()
            .for_each(|f| debug!("[device/features/#] - {:?}", f));

        app.on_request_features(&mut requested_features);

        trace!("[device/features] Requested features:");
        requested_features
            .iter()
            .for_each(|f| debug!("[device/features/#] - {:?}", f));

        let available_features = FeatureStructs::available(instance, physical_device);
        let available_features_list = available_features.get_list();
        trace!("[device/features] Available features:");
        available_features_list
            .iter()
            .for_each(|f| trace!("[device/features/#] - {:?}", f));

        let mut device_features_sets =
            FeatureStructs::validate_and_write(available_features, requested_features.as_slice())?;

        let resolved_features_list = device_features_sets.get_list();

        debug!("[device/features] Resolved features:");
        resolved_features_list
            .iter()
            .for_each(|f| debug!("[device/features/#] - {:?}", f));

        app.on_resolve_features(&device_features_sets);

        let mut device_features = device_features_sets.make_features_2();

        let create_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(extensions.as_slice())
            .queue_create_infos(device_queue_create_infos.as_slice())
            .push_next(&mut device_features);

        let device = unsafe { instance.create_device(physical_device, &create_info, None) }?;

        info!("[vulkan/device] Successfully created device");

        let queues = device_queue_create_infos
            .iter()
            .cloned()
            .map(|ci| (ci.queue_family_index, ci.queue_count))
            .map(|(family, count)| {
                (
                    family,
                    (0..count)
                        .map(|i| unsafe { device.get_device_queue(family, i) })
                        .collect::<Vec<vk::Queue>>(),
                )
            })
            .collect::<HashMap<u32, Vec<vk::Queue>>>();

        info!(
            "[device/queues] Successfully loaded {:?} device queues",
            queues.iter().fold(0usize, |a, (_, v)| a + v.len())
        );

        let device_loader = DeviceLoader::load(&instance, &device);

        Ok(Device {
            device,
            queues,
            queue_labels: labeled,
            unlabeled_queues: unlabeled,
            loader: device_loader,
        })
    }

    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    pub fn queues(&self) -> &HashMap<u32, Vec<vk::Queue>> {
        &self.queues
    }

    pub fn queue_labels(&self) -> &QueueLabels {
        &self.queue_labels
    }

    pub fn unlabeled_queues(&self) -> &UnlabeledQueues {
        &self.unlabeled_queues
    }

    pub fn loader(&self) -> &DeviceLoader {
        &self.loader
    }

    pub fn get_labeled_queue_ref(&self, label: QueueLabel) -> Option<QueueRef> {
        self.queue_labels.get(&label).and_then(|v| v.first()).cloned()
    }

    pub fn get_labeled_queue(&self, label: QueueLabel) -> Option<vk::Queue> {
        self.queue_labels.get(&label).and_then(|v| v.first()).and_then(|qr| self.get_queue(qr.clone()))
    }

    pub fn get_queue(&self, queue_ref: QueueRef) -> Option<vk::Queue> {
        self.queues.get(&queue_ref.family).and_then(|queues| queues.get(queue_ref.index as usize)).cloned()
    }

    pub fn wait_idle(&self) -> VkResult<()> {
        unsafe { self.device.device_wait_idle() }
    }

    pub fn wait_queues(&self, family: u32) -> VkResult<()> {
        unsafe {
            if let Some(queues) = self.queues.get(&family) {
                for q in queues {
                    self.queue_wait_idle(q.clone())?;
                }
            }
        }

        Ok(())
    }
}

impl Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DerefMut for Device {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

pub struct DeviceLoader {
    swapchain: khr::swapchain::Device,
}


impl DeviceLoader {
    pub fn load(instance: &ash::Instance, device: &ash::Device) -> Self {
        Self {
            swapchain: khr::swapchain::Device::new(instance, device),
        }
    }

    pub fn swapchain(&self) -> &khr::swapchain::Device {
        &self.swapchain
    }
}
