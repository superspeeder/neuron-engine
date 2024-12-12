use anyhow::anyhow;
use log::info;
use neuron_engine::app::feature_request::{ExtensionRequest, QueueRequest, RequestHelper};
use neuron_engine::app::{Application, run};
use neuron_engine::ash::vk::{CommandBufferResetFlags, QueueFamilyProperties};
use neuron_engine::ash::{ext, vk};
use neuron_engine::render::context::VulkanContext;
use neuron_engine::render::context::command_pool::CommandPool;
use neuron_engine::render::context::device::Device;
use neuron_engine::render::context::instance::Instance;
use neuron_engine::render::context::queues::QueueLabel;
use neuron_engine::render::frame_set::FrameSet;
use neuron_engine::winit::event_loop::ActiveEventLoop;
use neuron_engine::winit::window::{Window, WindowId};
use neuron_engine::{Engine, EngineCallbackHandler};
use std::sync::Arc;

pub const NAME: &str = "Neuron Example Application";

struct State {
    vulkan_context: Arc<VulkanContext>,
    command_pool: CommandPool,
    command_buffers: FrameSet<vk::CommandBuffer>,
    graphics_queue: vk::Queue,
    graphics_queue_family: u32,
}

impl State {
    fn new(engine: &Engine) -> anyhow::Result<State> {
        let Some(queue_ref) = engine
            .vulkan()
            .device()
            .get_labeled_queue_ref(QueueLabel::Graphics)
        else {
            return Err(anyhow!("Failed to get graphics queue"));
        };

        let command_pool = CommandPool::new(engine.vulkan(), queue_ref.family, true)?;
        let command_buffers = command_pool.allocate_command_buffer_set()?;
        let graphics_queue = engine
            .vulkan()
            .device()
            .get_queue(queue_ref.clone())
            .expect("Invalid graphics queue ref returned by engine");

        Ok(State {
            vulkan_context: engine.vulkan(),
            command_pool,
            command_buffers,
            graphics_queue,
            graphics_queue_family: queue_ref.family,
        })
    }
}

struct MyApp {
    state: Option<State>,
}

impl EngineCallbackHandler for MyApp {
    fn name(&self) -> &str {
        NAME
    }

    fn on_request_device_extensions(&mut self, requests: &mut Vec<ExtensionRequest>) {
        requests
            .required(ext::extended_dynamic_state3::NAME)
            .optional(ext::image_2d_view_of_3d::NAME);
    }

    fn on_physical_device(&mut self, physical_device: vk::PhysicalDevice, instance: &Instance) {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        info!("Physical Device Type: {:?}", properties.device_type);
    }

    fn on_queue_selection(
        &mut self,
        _existing_requests: &[QueueRequest],
        _families: Vec<QueueFamilyProperties>,
    ) -> anyhow::Result<Vec<QueueRequest>> {
        Ok(vec![
            QueueRequest::flexible_labeled(1, 2, QueueLabel::Transfer),
            QueueRequest::flexible_labeled_custom(0, 22, "Graphics Extras"),
            QueueRequest::strict_labeled_custom(0, 1, "Graphics Exclusive"),
        ])
    }

    fn on_engine_ready(&mut self, engine: &mut Engine) -> anyhow::Result<()> {
        self.state = Some(State::new(engine)?);

        Ok(())
    }
}

impl Application for MyApp {
    fn on_create_windows(&mut self, event_loop: &ActiveEventLoop, engine: &mut Engine) {
        _ = engine.create_window(
            event_loop,
            Window::default_attributes().with_title(self.name()),
        );
    }

    fn on_redraw_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        engine: &mut Engine,
    ) {
        let Some(state) = self.state.as_ref() else {
            return;
        };

        if let Some(window) = engine.get_window(&window_id) {
            window
                .borrow_mut()
                .render_frame(|window, image| {
                    let command_buffer = state.command_buffers[image.current_frame()].clone();
                    let vulkan = engine.vulkan();
                    let device = vulkan.device();

                    unsafe {
                        device.reset_command_buffer(
                            command_buffer,
                            CommandBufferResetFlags::empty(),
                        )?;
                        device.begin_command_buffer(
                            command_buffer,
                            &vk::CommandBufferBeginInfo::default()
                                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                        )?;

                        let image_barrier1 = vk::ImageMemoryBarrier::default()
                            .image(image.image())
                            .src_access_mask(vk::AccessFlags::empty())
                            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                            .src_queue_family_index(image.present_queue_family())
                            .dst_queue_family_index(state.graphics_queue_family)
                            .old_layout(vk::ImageLayout::UNDEFINED)
                            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                            .subresource_range(
                                vk::ImageSubresourceRange::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .base_array_layer(0)
                                    .layer_count(1)
                                    .base_mip_level(0)
                                    .level_count(1),
                            );

                        let image_barrier2 = vk::ImageMemoryBarrier::default()
                            .image(image.image())
                            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                            .dst_access_mask(vk::AccessFlags::empty())
                            .src_queue_family_index(state.graphics_queue_family)
                            .dst_queue_family_index(image.present_queue_family())
                            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                            .subresource_range(
                                vk::ImageSubresourceRange::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .base_array_layer(0)
                                    .layer_count(1)
                                    .base_mip_level(0)
                                    .level_count(1),
                            );

                        device.cmd_pipeline_barrier(
                            command_buffer,
                            vk::PipelineStageFlags::TOP_OF_PIPE,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &[image_barrier1],
                        );

                        let mut color = vk::ClearColorValue::default();
                        color.float32 = [1.0f32, 0.0f32, 0.0f32, 1.0f32];

                        device.cmd_clear_color_image(
                            command_buffer,
                            image.image(),
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            &color,
                            &[vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_array_layer(0)
                                .layer_count(1)
                                .base_mip_level(0)
                                .level_count(1)],
                        );

                        device.cmd_pipeline_barrier(
                            command_buffer,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &[image_barrier2],
                        );

                        device.end_command_buffer(command_buffer)?;

                        let cmds = [command_buffer];
                        let waits = [image.image_available_semaphore()];
                        let signals = [image.render_finished_semaphore()];
                        let wait_stages = [vk::PipelineStageFlags::TOP_OF_PIPE];

                        let submit_info = vk::SubmitInfo::default()
                            .command_buffers(&cmds)
                            .wait_semaphores(&waits)
                            .wait_dst_stage_mask(&wait_stages)
                            .signal_semaphores(&signals);

                        device.queue_submit(state.graphics_queue, &[submit_info], image.in_flight_fence())?;
                    }

                    Ok(())
                })
                .expect("Failed to render frame");
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let app = MyApp { state: None };
    run(app)
}
