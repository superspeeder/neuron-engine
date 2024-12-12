use std::collections::HashSet;
use std::ffi::CStr;
use anyhow::anyhow;
use ash::vk;
use crate::render::context::queues::QueueLabel;

#[derive(Default)]
pub struct FeatureStructs<'a> {
    features1: vk::PhysicalDeviceFeatures,
    vk11: vk::PhysicalDeviceVulkan11Features<'a>,
    vk12: vk::PhysicalDeviceVulkan12Features<'a>,
    vk13: vk::PhysicalDeviceVulkan13Features<'a>,
}

impl<'a> FeatureStructs<'a> {
    pub(crate) fn validate_and_write<'b>(
        support: FeatureStructs<'b>,
        feature_requests: &[DeviceFeatureRequest],
    ) -> anyhow::Result<FeatureStructs<'a>> {
        let mut features = FeatureStructs::<'a>::default();

        for req in feature_requests {
            if support.supports(req.feature) {
                *features.feature_mut(req.feature) = vk::TRUE;
            } else if req.required {
                return Err(anyhow!("Missing required feature {:?}", req.feature));
            }
        }

        Ok(features)
    }

    fn feature_ref(&self, feature: DeviceFeature) -> &vk::Bool32 {
        match feature {
            DeviceFeature::RobustBufferAccess => &self.features1.robust_buffer_access,
            DeviceFeature::FullDrawIndexUint32 => &self.features1.full_draw_index_uint32,
            DeviceFeature::ImageCubeArray => &self.features1.image_cube_array,
            DeviceFeature::IndependentBlend => &self.features1.independent_blend,
            DeviceFeature::GeometryShader => &self.features1.geometry_shader,
            DeviceFeature::TessellationShader => &self.features1.tessellation_shader,
            DeviceFeature::SampleRateShading => &self.features1.sample_rate_shading,
            DeviceFeature::DualSourceBlend => &self.features1.dual_src_blend,
            DeviceFeature::LogicOperation => &self.features1.logic_op,
            DeviceFeature::MultiDrawIndirect => &self.features1.multi_draw_indirect,
            DeviceFeature::WideLines => &self.features1.wide_lines,
            DeviceFeature::LargePoints => &self.features1.large_points,
            DeviceFeature::AlphaToOne => &self.features1.alpha_to_one,
            DeviceFeature::MultiViewport => &self.features1.multi_viewport,
            DeviceFeature::SamplerAnisotropy => &self.features1.sampler_anisotropy,
            DeviceFeature::TextureCompressionETC2 => &self.features1.texture_compression_etc2,
            DeviceFeature::TextureCompressionASTCLDR => {
                &self.features1.texture_compression_astc_ldr
            }
            DeviceFeature::TextureCompressionBC => &self.features1.texture_compression_bc,
            DeviceFeature::OcclusionQueryPrecise => &self.features1.occlusion_query_precise,
            DeviceFeature::PipelineStatisticsQuery => &self.features1.pipeline_statistics_query,
            DeviceFeature::VertexPipelineStoresAndAtomics => {
                &self.features1.vertex_pipeline_stores_and_atomics
            }
            DeviceFeature::FragmentStoresAndAtomics => &self.features1.fragment_stores_and_atomics,
            DeviceFeature::ShaderTessellationAndGeometryPointSize => {
                &self.features1.shader_tessellation_and_geometry_point_size
            }
            DeviceFeature::ShaderImageGatherExtended => {
                &self.features1.shader_image_gather_extended
            }
            DeviceFeature::ShaderStorageImageExtendedFormats => {
                &self.features1.shader_storage_image_extended_formats
            }
            DeviceFeature::ShaderStorageImageMultisample => {
                &self.features1.shader_storage_image_multisample
            }
            DeviceFeature::ShaderStorageImageReadWithoutFormat => {
                &self.features1.shader_storage_image_read_without_format
            }
            DeviceFeature::ShaderStorageImageWriteWithoutFormat => {
                &self.features1.shader_storage_image_write_without_format
            }
            DeviceFeature::ShaderUniformBufferArrayDynamicIndexing => {
                &self.features1.shader_uniform_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderSampledImageArrayDynamicIndexing => {
                &self.features1.shader_sampled_image_array_dynamic_indexing
            }
            DeviceFeature::ShaderStorageBufferArrayDynamicIndexing => {
                &self.features1.shader_storage_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderStorageImageArrayDynamicIndexing => {
                &self.features1.shader_storage_image_array_dynamic_indexing
            }
            DeviceFeature::ShaderClipDistance => &self.features1.shader_clip_distance,
            DeviceFeature::ShaderCullDistance => &self.features1.shader_cull_distance,
            DeviceFeature::ShaderFloat64 => &self.features1.shader_float64,
            DeviceFeature::ShaderInt64 => &self.features1.shader_int64,
            DeviceFeature::ShaderInt16 => &self.features1.shader_int16,
            DeviceFeature::ShaderResourceResidency => &self.features1.shader_resource_residency,
            DeviceFeature::ShaderResourceMinLod => &self.features1.shader_resource_min_lod,
            DeviceFeature::SparseBinding => &self.features1.sparse_binding,
            DeviceFeature::SparseResidencyBuffer => &self.features1.sparse_residency_buffer,
            DeviceFeature::SparseResidencyImage2D => &self.features1.sparse_residency_image2_d,
            DeviceFeature::SparseResidencyImage3D => &self.features1.sparse_residency_image3_d,
            DeviceFeature::SparseResidency2Samples => &self.features1.sparse_residency2_samples,
            DeviceFeature::SparseResidency4Samples => &self.features1.sparse_residency4_samples,
            DeviceFeature::SparseResidency8Samples => &self.features1.sparse_residency8_samples,
            DeviceFeature::SparseResidency16Samples => &self.features1.sparse_residency16_samples,
            DeviceFeature::VariableMultisampleRate => &self.features1.variable_multisample_rate,
            DeviceFeature::InheritedQueries => &self.features1.inherited_queries,
            DeviceFeature::StorageBuffer16BitAccess => &self.vk11.storage_buffer16_bit_access,
            DeviceFeature::UniformAndStorageBuffer16BitAccess => {
                &self.vk11.uniform_and_storage_buffer16_bit_access
            }
            DeviceFeature::StoragePushConstant16 => &self.vk11.storage_push_constant16,
            DeviceFeature::StorageInputOutput16 => &self.vk11.storage_input_output16,
            DeviceFeature::Multiview => &self.vk11.multiview,
            DeviceFeature::MultiviewGeometryShader => &self.vk11.multiview_geometry_shader,
            DeviceFeature::MultiviewTessellationShader => &self.vk11.multiview_tessellation_shader,
            DeviceFeature::VariablePointersStorageBuffer => {
                &self.vk11.variable_pointers_storage_buffer
            }
            DeviceFeature::VariablePointers => &self.vk11.variable_pointers,
            DeviceFeature::ProtectedMemory => &self.vk11.protected_memory,
            DeviceFeature::SamplerYcbcrConversion => &self.vk11.sampler_ycbcr_conversion,
            DeviceFeature::ShaderDrawParameters => &self.vk11.shader_draw_parameters,
            DeviceFeature::SamplerMirrorClampToEdge => &self.vk12.sampler_mirror_clamp_to_edge,
            DeviceFeature::DrawIndirectCount => &self.vk12.draw_indirect_count,
            DeviceFeature::StorageBuffer8BitAccess => &self.vk12.storage_buffer8_bit_access,
            DeviceFeature::UniformAndStorageBuffer8BitAccess => {
                &self.vk12.uniform_and_storage_buffer8_bit_access
            }
            DeviceFeature::ShaderBufferInt64Atomics => &self.vk12.shader_buffer_int64_atomics,
            DeviceFeature::ShaderSharedInt64Atomics => &self.vk12.shader_shared_int64_atomics,
            DeviceFeature::ShaderFloat16 => &self.vk12.shader_float16,
            DeviceFeature::ShaderInt8 => &self.vk12.shader_int8,
            DeviceFeature::DescriptorIndexing => &self.vk12.descriptor_indexing,
            DeviceFeature::ShaderInputAttachmentArrayDynamicIndexing => {
                &self.vk12.shader_input_attachment_array_dynamic_indexing
            }
            DeviceFeature::ShaderUniformTexelBufferArrayDynamicIndexing => {
                &self.vk12.shader_uniform_texel_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderStorageTexelBufferArrayDynamicIndexing => {
                &self.vk12.shader_storage_texel_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderUniformBufferArrayNonUniformIndexing => {
                &self.vk12.shader_uniform_buffer_array_non_uniform_indexing
            }
            DeviceFeature::ShaderSampledImageArrayNonUniformIndexing => {
                &self.vk12.shader_sampled_image_array_non_uniform_indexing
            }
            DeviceFeature::ShaderStorageBufferArrayNonUniformIndexing => {
                &self.vk12.shader_storage_buffer_array_non_uniform_indexing
            }
            DeviceFeature::ShaderStorageImageArrayNonUniformIndexing => {
                &self.vk12.shader_storage_image_array_non_uniform_indexing
            }
            DeviceFeature::ShaderInputAttachmentArrayNonUniformIndexing => {
                &self.vk12.shader_input_attachment_array_non_uniform_indexing
            }
            DeviceFeature::ShaderUniformTexelBufferArrayNonUniformIndexing => {
                &self
                    .vk12
                    .shader_uniform_texel_buffer_array_non_uniform_indexing
            }
            DeviceFeature::ShaderStorageTexelBufferArrayNonUniformIndexing => {
                &self.vk12.shader_storage_buffer_array_non_uniform_indexing
            }
            DeviceFeature::DescriptorBindingUniformBufferUpdateAfterBind => {
                &self
                    .vk12
                    .descriptor_binding_uniform_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingSampledImageUpdateAfterBind => {
                &self.vk12.descriptor_binding_sampled_image_update_after_bind
            }
            DeviceFeature::DescriptorBindingStorageImageUpdateAfterBind => {
                &self.vk12.descriptor_binding_storage_image_update_after_bind
            }
            DeviceFeature::DescriptorBindingStorageBufferUpdateAfterBind => {
                &self
                    .vk12
                    .descriptor_binding_storage_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingUniformTexelBufferUpdateAfterBind => {
                &self
                    .vk12
                    .descriptor_binding_uniform_texel_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingStorageTexelBufferUpdateAfterBind => {
                &self
                    .vk12
                    .descriptor_binding_storage_texel_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingUpdateUnusedWhilePending => {
                &self.vk12.descriptor_binding_update_unused_while_pending
            }
            DeviceFeature::DescriptorBindingPartiallyBound => {
                &self.vk12.descriptor_binding_partially_bound
            }
            DeviceFeature::DescriptorBindingVariableDescriptorCount => {
                &self.vk12.descriptor_binding_variable_descriptor_count
            }
            DeviceFeature::RuntimeDescriptorArray => &self.vk12.runtime_descriptor_array,
            DeviceFeature::SamplerFilterMinmax => &self.vk12.sampler_filter_minmax,
            DeviceFeature::ScalarBlockLayout => &self.vk12.scalar_block_layout,
            DeviceFeature::ImagelessFramebuffer => &self.vk12.imageless_framebuffer,
            DeviceFeature::UniformBufferStandardLayout => &self.vk12.uniform_buffer_standard_layout,
            DeviceFeature::ShaderSubgroupExtendedTypes => &self.vk12.shader_subgroup_extended_types,
            DeviceFeature::SeparateDepthStencilLayouts => &self.vk12.separate_depth_stencil_layouts,
            DeviceFeature::HostQueryReset => &self.vk12.host_query_reset,
            DeviceFeature::TimelineSemaphore => &self.vk12.timeline_semaphore,
            DeviceFeature::BufferDeviceAddress => &self.vk12.buffer_device_address,
            DeviceFeature::BufferDeviceAddressCaptureReplay => {
                &self.vk12.buffer_device_address_capture_replay
            }
            DeviceFeature::BufferDeviceAddressMultiDevice => {
                &self.vk12.buffer_device_address_multi_device
            }
            DeviceFeature::VulkanMemoryModel => &self.vk12.vulkan_memory_model,
            DeviceFeature::VulkanMemoryModelDeviceScope => {
                &self.vk12.vulkan_memory_model_device_scope
            }
            DeviceFeature::VulkanMemoryModelAvailabilityVisibilityChains => {
                &self.vk12.vulkan_memory_model_availability_visibility_chains
            }
            DeviceFeature::ShaderOutputViewportIndex => &self.vk12.shader_output_viewport_index,
            DeviceFeature::ShaderOutputLayer => &self.vk12.shader_output_layer,
            DeviceFeature::SubgroupBroadcastDynamicId => &self.vk12.subgroup_broadcast_dynamic_id,
            DeviceFeature::RobustImageAccess => &self.vk13.robust_image_access,
            DeviceFeature::InlineUniformBlock => &self.vk13.inline_uniform_block,
            DeviceFeature::DescriptorBindingInlineUniformBlockUpdateAfterBind => {
                &self
                    .vk13
                    .descriptor_binding_inline_uniform_block_update_after_bind
            }
            DeviceFeature::PipelineCreationCacheControl => {
                &self.vk13.pipeline_creation_cache_control
            }
            DeviceFeature::PrivateData => &self.vk13.private_data,
            DeviceFeature::ShaderDemoteToHelperInvocation => {
                &self.vk13.shader_demote_to_helper_invocation
            }
            DeviceFeature::ShaderTerminateInvocation => &self.vk13.shader_terminate_invocation,
            DeviceFeature::ComputeFullSubgroups => &self.vk13.compute_full_subgroups,
            DeviceFeature::Synchronization2 => &self.vk13.synchronization2,
            DeviceFeature::TextureCompressionASTCHDR => &self.vk13.texture_compression_astc_hdr,
            DeviceFeature::ShaderZeroInitializeWorkgroupMemory => {
                &self.vk13.shader_zero_initialize_workgroup_memory
            }
            DeviceFeature::DynamicRendering => &self.vk13.dynamic_rendering,
            DeviceFeature::ShaderIntegerDotProduct => &self.vk13.shader_integer_dot_product,
            DeviceFeature::Maintenance4 => &self.vk13.maintenance4,
        }
    }

    fn feature_mut(&mut self, feature: DeviceFeature) -> &mut vk::Bool32 {
        match feature {
            DeviceFeature::RobustBufferAccess => &mut self.features1.robust_buffer_access,
            DeviceFeature::FullDrawIndexUint32 => &mut self.features1.full_draw_index_uint32,
            DeviceFeature::ImageCubeArray => &mut self.features1.image_cube_array,
            DeviceFeature::IndependentBlend => &mut self.features1.independent_blend,
            DeviceFeature::GeometryShader => &mut self.features1.geometry_shader,
            DeviceFeature::TessellationShader => &mut self.features1.tessellation_shader,
            DeviceFeature::SampleRateShading => &mut self.features1.sample_rate_shading,
            DeviceFeature::DualSourceBlend => &mut self.features1.dual_src_blend,
            DeviceFeature::LogicOperation => &mut self.features1.logic_op,
            DeviceFeature::MultiDrawIndirect => &mut self.features1.multi_draw_indirect,
            DeviceFeature::WideLines => &mut self.features1.wide_lines,
            DeviceFeature::LargePoints => &mut self.features1.large_points,
            DeviceFeature::AlphaToOne => &mut self.features1.alpha_to_one,
            DeviceFeature::MultiViewport => &mut self.features1.multi_viewport,
            DeviceFeature::SamplerAnisotropy => &mut self.features1.sampler_anisotropy,
            DeviceFeature::TextureCompressionETC2 => &mut self.features1.texture_compression_etc2,
            DeviceFeature::TextureCompressionASTCLDR => {
                &mut self.features1.texture_compression_astc_ldr
            }
            DeviceFeature::TextureCompressionBC => &mut self.features1.texture_compression_bc,
            DeviceFeature::OcclusionQueryPrecise => &mut self.features1.occlusion_query_precise,
            DeviceFeature::PipelineStatisticsQuery => &mut self.features1.pipeline_statistics_query,
            DeviceFeature::VertexPipelineStoresAndAtomics => {
                &mut self.features1.vertex_pipeline_stores_and_atomics
            }
            DeviceFeature::FragmentStoresAndAtomics => {
                &mut self.features1.fragment_stores_and_atomics
            }
            DeviceFeature::ShaderTessellationAndGeometryPointSize => {
                &mut self.features1.shader_tessellation_and_geometry_point_size
            }
            DeviceFeature::ShaderImageGatherExtended => {
                &mut self.features1.shader_image_gather_extended
            }
            DeviceFeature::ShaderStorageImageExtendedFormats => {
                &mut self.features1.shader_storage_image_extended_formats
            }
            DeviceFeature::ShaderStorageImageMultisample => {
                &mut self.features1.shader_storage_image_multisample
            }
            DeviceFeature::ShaderStorageImageReadWithoutFormat => {
                &mut self.features1.shader_storage_image_read_without_format
            }
            DeviceFeature::ShaderStorageImageWriteWithoutFormat => {
                &mut self.features1.shader_storage_image_write_without_format
            }
            DeviceFeature::ShaderUniformBufferArrayDynamicIndexing => {
                &mut self.features1.shader_uniform_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderSampledImageArrayDynamicIndexing => {
                &mut self.features1.shader_sampled_image_array_dynamic_indexing
            }
            DeviceFeature::ShaderStorageBufferArrayDynamicIndexing => {
                &mut self.features1.shader_storage_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderStorageImageArrayDynamicIndexing => {
                &mut self.features1.shader_storage_image_array_dynamic_indexing
            }
            DeviceFeature::ShaderClipDistance => &mut self.features1.shader_clip_distance,
            DeviceFeature::ShaderCullDistance => &mut self.features1.shader_cull_distance,
            DeviceFeature::ShaderFloat64 => &mut self.features1.shader_float64,
            DeviceFeature::ShaderInt64 => &mut self.features1.shader_int64,
            DeviceFeature::ShaderInt16 => &mut self.features1.shader_int16,
            DeviceFeature::ShaderResourceResidency => &mut self.features1.shader_resource_residency,
            DeviceFeature::ShaderResourceMinLod => &mut self.features1.shader_resource_min_lod,
            DeviceFeature::SparseBinding => &mut self.features1.sparse_binding,
            DeviceFeature::SparseResidencyBuffer => &mut self.features1.sparse_residency_buffer,
            DeviceFeature::SparseResidencyImage2D => &mut self.features1.sparse_residency_image2_d,
            DeviceFeature::SparseResidencyImage3D => &mut self.features1.sparse_residency_image3_d,
            DeviceFeature::SparseResidency2Samples => &mut self.features1.sparse_residency2_samples,
            DeviceFeature::SparseResidency4Samples => &mut self.features1.sparse_residency4_samples,
            DeviceFeature::SparseResidency8Samples => &mut self.features1.sparse_residency8_samples,
            DeviceFeature::SparseResidency16Samples => {
                &mut self.features1.sparse_residency16_samples
            }
            DeviceFeature::VariableMultisampleRate => &mut self.features1.variable_multisample_rate,
            DeviceFeature::InheritedQueries => &mut self.features1.inherited_queries,
            DeviceFeature::StorageBuffer16BitAccess => &mut self.vk11.storage_buffer16_bit_access,
            DeviceFeature::UniformAndStorageBuffer16BitAccess => {
                &mut self.vk11.uniform_and_storage_buffer16_bit_access
            }
            DeviceFeature::StoragePushConstant16 => &mut self.vk11.storage_push_constant16,
            DeviceFeature::StorageInputOutput16 => &mut self.vk11.storage_input_output16,
            DeviceFeature::Multiview => &mut self.vk11.multiview,
            DeviceFeature::MultiviewGeometryShader => &mut self.vk11.multiview_geometry_shader,
            DeviceFeature::MultiviewTessellationShader => {
                &mut self.vk11.multiview_tessellation_shader
            }
            DeviceFeature::VariablePointersStorageBuffer => {
                &mut self.vk11.variable_pointers_storage_buffer
            }
            DeviceFeature::VariablePointers => &mut self.vk11.variable_pointers,
            DeviceFeature::ProtectedMemory => &mut self.vk11.protected_memory,
            DeviceFeature::SamplerYcbcrConversion => &mut self.vk11.sampler_ycbcr_conversion,
            DeviceFeature::ShaderDrawParameters => &mut self.vk11.shader_draw_parameters,
            DeviceFeature::SamplerMirrorClampToEdge => &mut self.vk12.sampler_mirror_clamp_to_edge,
            DeviceFeature::DrawIndirectCount => &mut self.vk12.draw_indirect_count,
            DeviceFeature::StorageBuffer8BitAccess => &mut self.vk12.storage_buffer8_bit_access,
            DeviceFeature::UniformAndStorageBuffer8BitAccess => {
                &mut self.vk12.uniform_and_storage_buffer8_bit_access
            }
            DeviceFeature::ShaderBufferInt64Atomics => &mut self.vk12.shader_buffer_int64_atomics,
            DeviceFeature::ShaderSharedInt64Atomics => &mut self.vk12.shader_shared_int64_atomics,
            DeviceFeature::ShaderFloat16 => &mut self.vk12.shader_float16,
            DeviceFeature::ShaderInt8 => &mut self.vk12.shader_int8,
            DeviceFeature::DescriptorIndexing => &mut self.vk12.descriptor_indexing,
            DeviceFeature::ShaderInputAttachmentArrayDynamicIndexing => {
                &mut self.vk12.shader_input_attachment_array_dynamic_indexing
            }
            DeviceFeature::ShaderUniformTexelBufferArrayDynamicIndexing => {
                &mut self.vk12.shader_uniform_texel_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderStorageTexelBufferArrayDynamicIndexing => {
                &mut self.vk12.shader_storage_texel_buffer_array_dynamic_indexing
            }
            DeviceFeature::ShaderUniformBufferArrayNonUniformIndexing => {
                &mut self.vk12.shader_uniform_buffer_array_non_uniform_indexing
            }
            DeviceFeature::ShaderSampledImageArrayNonUniformIndexing => {
                &mut self.vk12.shader_sampled_image_array_non_uniform_indexing
            }
            DeviceFeature::ShaderStorageBufferArrayNonUniformIndexing => {
                &mut self.vk12.shader_storage_buffer_array_non_uniform_indexing
            }
            DeviceFeature::ShaderStorageImageArrayNonUniformIndexing => {
                &mut self.vk12.shader_storage_image_array_non_uniform_indexing
            }
            DeviceFeature::ShaderInputAttachmentArrayNonUniformIndexing => {
                &mut self.vk12.shader_input_attachment_array_non_uniform_indexing
            }
            DeviceFeature::ShaderUniformTexelBufferArrayNonUniformIndexing => {
                &mut self
                    .vk12
                    .shader_uniform_texel_buffer_array_non_uniform_indexing
            }
            DeviceFeature::ShaderStorageTexelBufferArrayNonUniformIndexing => {
                &mut self.vk12.shader_storage_buffer_array_non_uniform_indexing
            }
            DeviceFeature::DescriptorBindingUniformBufferUpdateAfterBind => {
                &mut self
                    .vk12
                    .descriptor_binding_uniform_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingSampledImageUpdateAfterBind => {
                &mut self.vk12.descriptor_binding_sampled_image_update_after_bind
            }
            DeviceFeature::DescriptorBindingStorageImageUpdateAfterBind => {
                &mut self.vk12.descriptor_binding_storage_image_update_after_bind
            }
            DeviceFeature::DescriptorBindingStorageBufferUpdateAfterBind => {
                &mut self
                    .vk12
                    .descriptor_binding_storage_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingUniformTexelBufferUpdateAfterBind => {
                &mut self
                    .vk12
                    .descriptor_binding_uniform_texel_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingStorageTexelBufferUpdateAfterBind => {
                &mut self
                    .vk12
                    .descriptor_binding_storage_texel_buffer_update_after_bind
            }
            DeviceFeature::DescriptorBindingUpdateUnusedWhilePending => {
                &mut self.vk12.descriptor_binding_update_unused_while_pending
            }
            DeviceFeature::DescriptorBindingPartiallyBound => {
                &mut self.vk12.descriptor_binding_partially_bound
            }
            DeviceFeature::DescriptorBindingVariableDescriptorCount => {
                &mut self.vk12.descriptor_binding_variable_descriptor_count
            }
            DeviceFeature::RuntimeDescriptorArray => &mut self.vk12.runtime_descriptor_array,
            DeviceFeature::SamplerFilterMinmax => &mut self.vk12.sampler_filter_minmax,
            DeviceFeature::ScalarBlockLayout => &mut self.vk12.scalar_block_layout,
            DeviceFeature::ImagelessFramebuffer => &mut self.vk12.imageless_framebuffer,
            DeviceFeature::UniformBufferStandardLayout => {
                &mut self.vk12.uniform_buffer_standard_layout
            }
            DeviceFeature::ShaderSubgroupExtendedTypes => {
                &mut self.vk12.shader_subgroup_extended_types
            }
            DeviceFeature::SeparateDepthStencilLayouts => {
                &mut self.vk12.separate_depth_stencil_layouts
            }
            DeviceFeature::HostQueryReset => &mut self.vk12.host_query_reset,
            DeviceFeature::TimelineSemaphore => &mut self.vk12.timeline_semaphore,
            DeviceFeature::BufferDeviceAddress => &mut self.vk12.buffer_device_address,
            DeviceFeature::BufferDeviceAddressCaptureReplay => {
                &mut self.vk12.buffer_device_address_capture_replay
            }
            DeviceFeature::BufferDeviceAddressMultiDevice => {
                &mut self.vk12.buffer_device_address_multi_device
            }
            DeviceFeature::VulkanMemoryModel => &mut self.vk12.vulkan_memory_model,
            DeviceFeature::VulkanMemoryModelDeviceScope => {
                &mut self.vk12.vulkan_memory_model_device_scope
            }
            DeviceFeature::VulkanMemoryModelAvailabilityVisibilityChains => {
                &mut self.vk12.vulkan_memory_model_availability_visibility_chains
            }
            DeviceFeature::ShaderOutputViewportIndex => &mut self.vk12.shader_output_viewport_index,
            DeviceFeature::ShaderOutputLayer => &mut self.vk12.shader_output_layer,
            DeviceFeature::SubgroupBroadcastDynamicId => {
                &mut self.vk12.subgroup_broadcast_dynamic_id
            }
            DeviceFeature::RobustImageAccess => &mut self.vk13.robust_image_access,
            DeviceFeature::InlineUniformBlock => &mut self.vk13.inline_uniform_block,
            DeviceFeature::DescriptorBindingInlineUniformBlockUpdateAfterBind => {
                &mut self
                    .vk13
                    .descriptor_binding_inline_uniform_block_update_after_bind
            }
            DeviceFeature::PipelineCreationCacheControl => {
                &mut self.vk13.pipeline_creation_cache_control
            }
            DeviceFeature::PrivateData => &mut self.vk13.private_data,
            DeviceFeature::ShaderDemoteToHelperInvocation => {
                &mut self.vk13.shader_demote_to_helper_invocation
            }
            DeviceFeature::ShaderTerminateInvocation => &mut self.vk13.shader_terminate_invocation,
            DeviceFeature::ComputeFullSubgroups => &mut self.vk13.compute_full_subgroups,
            DeviceFeature::Synchronization2 => &mut self.vk13.synchronization2,
            DeviceFeature::TextureCompressionASTCHDR => &mut self.vk13.texture_compression_astc_hdr,
            DeviceFeature::ShaderZeroInitializeWorkgroupMemory => {
                &mut self.vk13.shader_zero_initialize_workgroup_memory
            }
            DeviceFeature::DynamicRendering => &mut self.vk13.dynamic_rendering,
            DeviceFeature::ShaderIntegerDotProduct => &mut self.vk13.shader_integer_dot_product,
            DeviceFeature::Maintenance4 => &mut self.vk13.maintenance4,
        }
    }

    pub fn supports(&self, feature: DeviceFeature) -> bool {
        self.feature_ref(feature).clone() == vk::TRUE
    }

    pub fn available(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Self {
        let mut feature_struct = Self::default();
        let mut features2 = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut feature_struct.vk11)
            .push_next(&mut feature_struct.vk12)
            .push_next(&mut feature_struct.vk13);

        unsafe { instance.get_physical_device_features2(physical_device, &mut features2) };

        feature_struct.features1 = features2.features;
        feature_struct
    }

    pub(crate) fn make_features_2(&mut self) -> vk::PhysicalDeviceFeatures2 {
        vk::PhysicalDeviceFeatures2::default()
            .features(self.features1)
            .push_next(&mut self.vk11)
            .push_next(&mut self.vk12)
            .push_next(&mut self.vk13)
    }

    pub fn get_list(&self) -> HashSet<DeviceFeature> {
        let mut set = HashSet::new();

        if self.features1.robust_buffer_access == vk::TRUE {
            set.insert(DeviceFeature::RobustBufferAccess);
        }

        if self.features1.full_draw_index_uint32 == vk::TRUE {
            set.insert(DeviceFeature::FullDrawIndexUint32);
        }

        if self.features1.image_cube_array == vk::TRUE {
            set.insert(DeviceFeature::ImageCubeArray);
        }

        if self.features1.independent_blend == vk::TRUE {
            set.insert(DeviceFeature::IndependentBlend);
        }

        if self.features1.geometry_shader == vk::TRUE {
            set.insert(DeviceFeature::GeometryShader);
        }

        if self.features1.tessellation_shader == vk::TRUE {
            set.insert(DeviceFeature::TessellationShader);
        }

        if self.features1.sample_rate_shading == vk::TRUE {
            set.insert(DeviceFeature::SampleRateShading);
        }

        if self.features1.dual_src_blend == vk::TRUE {
            set.insert(DeviceFeature::DualSourceBlend);
        }

        if self.features1.logic_op == vk::TRUE {
            set.insert(DeviceFeature::LogicOperation);
        }

        if self.features1.multi_draw_indirect == vk::TRUE {
            set.insert(DeviceFeature::MultiDrawIndirect);
        }

        if self.features1.wide_lines == vk::TRUE {
            set.insert(DeviceFeature::WideLines);
        }

        if self.features1.large_points == vk::TRUE {
            set.insert(DeviceFeature::LargePoints);
        }

        if self.features1.alpha_to_one == vk::TRUE {
            set.insert(DeviceFeature::AlphaToOne);
        }

        if self.features1.multi_viewport == vk::TRUE {
            set.insert(DeviceFeature::MultiViewport);
        }

        if self.features1.sampler_anisotropy == vk::TRUE {
            set.insert(DeviceFeature::SamplerAnisotropy);
        }

        if self.features1.texture_compression_etc2 == vk::TRUE {
            set.insert(DeviceFeature::TextureCompressionETC2);
        }

        if self.features1.texture_compression_astc_ldr == vk::TRUE {
            set.insert(DeviceFeature::TextureCompressionASTCLDR);
        }

        if self.features1.texture_compression_bc == vk::TRUE {
            set.insert(DeviceFeature::TextureCompressionBC);
        }

        if self.features1.occlusion_query_precise == vk::TRUE {
            set.insert(DeviceFeature::OcclusionQueryPrecise);
        }

        if self.features1.pipeline_statistics_query == vk::TRUE {
            set.insert(DeviceFeature::PipelineStatisticsQuery);
        }

        if self.features1.vertex_pipeline_stores_and_atomics == vk::TRUE {
            set.insert(DeviceFeature::VertexPipelineStoresAndAtomics);
        }

        if self.features1.fragment_stores_and_atomics == vk::TRUE {
            set.insert(DeviceFeature::FragmentStoresAndAtomics);
        }

        if self.features1.shader_tessellation_and_geometry_point_size == vk::TRUE {
            set.insert(DeviceFeature::ShaderTessellationAndGeometryPointSize);
        }

        if self.features1.shader_image_gather_extended == vk::TRUE {
            set.insert(DeviceFeature::ShaderImageGatherExtended);
        }

        if self.features1.shader_storage_image_extended_formats == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageImageExtendedFormats);
        }

        if self.features1.shader_storage_image_multisample == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageImageMultisample);
        }

        if self.features1.shader_storage_image_read_without_format == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageImageReadWithoutFormat);
        }

        if self.features1.shader_storage_image_write_without_format == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageImageWriteWithoutFormat);
        }

        if self.features1.shader_uniform_buffer_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderUniformBufferArrayDynamicIndexing);
        }

        if self.features1.shader_sampled_image_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderSampledImageArrayDynamicIndexing);
        }

        if self.features1.shader_storage_buffer_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageBufferArrayDynamicIndexing);
        }

        if self.features1.shader_storage_image_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageImageArrayDynamicIndexing);
        }

        if self.features1.shader_clip_distance == vk::TRUE {
            set.insert(DeviceFeature::ShaderClipDistance);
        }

        if self.features1.shader_cull_distance == vk::TRUE {
            set.insert(DeviceFeature::ShaderCullDistance);
        }

        if self.features1.shader_float64 == vk::TRUE {
            set.insert(DeviceFeature::ShaderFloat64);
        }

        if self.features1.shader_int64 == vk::TRUE {
            set.insert(DeviceFeature::ShaderInt64);
        }

        if self.features1.shader_int16 == vk::TRUE {
            set.insert(DeviceFeature::ShaderInt16);
        }

        if self.features1.shader_resource_residency == vk::TRUE {
            set.insert(DeviceFeature::ShaderResourceResidency);
        }

        if self.features1.shader_resource_min_lod == vk::TRUE {
            set.insert(DeviceFeature::ShaderResourceMinLod);
        }

        if self.features1.sparse_binding == vk::TRUE {
            set.insert(DeviceFeature::SparseBinding);
        }

        if self.features1.sparse_residency_buffer == vk::TRUE {
            set.insert(DeviceFeature::SparseResidencyBuffer);
        }

        if self.features1.sparse_residency_image2_d == vk::TRUE {
            set.insert(DeviceFeature::SparseResidencyImage2D);
        }

        if self.features1.sparse_residency_image3_d == vk::TRUE {
            set.insert(DeviceFeature::SparseResidencyImage3D);
        }

        if self.features1.sparse_residency2_samples == vk::TRUE {
            set.insert(DeviceFeature::SparseResidency2Samples);
        }

        if self.features1.sparse_residency4_samples == vk::TRUE {
            set.insert(DeviceFeature::SparseResidency4Samples);
        }

        if self.features1.sparse_residency8_samples == vk::TRUE {
            set.insert(DeviceFeature::SparseResidency8Samples);
        }

        if self.features1.sparse_residency16_samples == vk::TRUE {
            set.insert(DeviceFeature::SparseResidency16Samples);
        }

        if self.features1.variable_multisample_rate == vk::TRUE {
            set.insert(DeviceFeature::VariableMultisampleRate);
        }

        if self.features1.inherited_queries == vk::TRUE {
            set.insert(DeviceFeature::InheritedQueries);
        }

        if self.vk11.storage_buffer16_bit_access == vk::TRUE {
            set.insert(DeviceFeature::StorageBuffer16BitAccess);
        }

        if self.vk11.uniform_and_storage_buffer16_bit_access == vk::TRUE {
            set.insert(DeviceFeature::UniformAndStorageBuffer16BitAccess);
        }

        if self.vk11.storage_push_constant16 == vk::TRUE {
            set.insert(DeviceFeature::StoragePushConstant16);
        }

        if self.vk11.storage_input_output16 == vk::TRUE {
            set.insert(DeviceFeature::StorageInputOutput16);
        }

        if self.vk11.multiview == vk::TRUE {
            set.insert(DeviceFeature::Multiview);
        }

        if self.vk11.multiview_geometry_shader == vk::TRUE {
            set.insert(DeviceFeature::MultiviewGeometryShader);
        }

        if self.vk11.multiview_tessellation_shader == vk::TRUE {
            set.insert(DeviceFeature::MultiviewTessellationShader);
        }

        if self.vk11.variable_pointers_storage_buffer == vk::TRUE {
            set.insert(DeviceFeature::VariablePointersStorageBuffer);
        }

        if self.vk11.variable_pointers == vk::TRUE {
            set.insert(DeviceFeature::VariablePointers);
        }

        if self.vk11.protected_memory == vk::TRUE {
            set.insert(DeviceFeature::ProtectedMemory);
        }

        if self.vk11.sampler_ycbcr_conversion == vk::TRUE {
            set.insert(DeviceFeature::SamplerYcbcrConversion);
        }

        if self.vk11.shader_draw_parameters == vk::TRUE {
            set.insert(DeviceFeature::ShaderDrawParameters);
        }

        if self.vk12.sampler_mirror_clamp_to_edge == vk::TRUE {
            set.insert(DeviceFeature::SamplerMirrorClampToEdge);
        }

        if self.vk12.draw_indirect_count == vk::TRUE {
            set.insert(DeviceFeature::DrawIndirectCount);
        }

        if self.vk12.storage_buffer8_bit_access == vk::TRUE {
            set.insert(DeviceFeature::StorageBuffer8BitAccess);
        }

        if self.vk12.uniform_and_storage_buffer8_bit_access == vk::TRUE {
            set.insert(DeviceFeature::UniformAndStorageBuffer8BitAccess);
        }

        if self.vk12.shader_buffer_int64_atomics == vk::TRUE {
            set.insert(DeviceFeature::ShaderBufferInt64Atomics);
        }

        if self.vk12.shader_shared_int64_atomics == vk::TRUE {
            set.insert(DeviceFeature::ShaderSharedInt64Atomics);
        }

        if self.vk12.shader_float16 == vk::TRUE {
            set.insert(DeviceFeature::ShaderFloat16);
        }

        if self.vk12.shader_int8 == vk::TRUE {
            set.insert(DeviceFeature::ShaderInt8);
        }

        if self.vk12.descriptor_indexing == vk::TRUE {
            set.insert(DeviceFeature::DescriptorIndexing);
        }

        if self.vk12.shader_input_attachment_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderInputAttachmentArrayDynamicIndexing);
        }

        if self.vk12.shader_uniform_texel_buffer_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderUniformTexelBufferArrayDynamicIndexing);
        }

        if self.vk12.shader_storage_texel_buffer_array_dynamic_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageTexelBufferArrayDynamicIndexing);
        }

        if self.vk12.shader_uniform_buffer_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderUniformBufferArrayNonUniformIndexing);
        }

        if self.vk12.shader_sampled_image_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderSampledImageArrayNonUniformIndexing);
        }

        if self.vk12.shader_storage_buffer_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageBufferArrayNonUniformIndexing);
        }

        if self.vk12.shader_storage_image_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageImageArrayNonUniformIndexing);
        }

        if self.vk12.shader_input_attachment_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderInputAttachmentArrayNonUniformIndexing);
        }

        if self.vk12.shader_uniform_texel_buffer_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderUniformTexelBufferArrayNonUniformIndexing);
        }

        if self.vk12.shader_storage_texel_buffer_array_non_uniform_indexing == vk::TRUE {
            set.insert(DeviceFeature::ShaderStorageTexelBufferArrayNonUniformIndexing);
        }

        if self.vk12.descriptor_binding_uniform_buffer_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingUniformBufferUpdateAfterBind);
        }

        if self.vk12.descriptor_binding_sampled_image_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingSampledImageUpdateAfterBind);
        }

        if self.vk12.descriptor_binding_storage_image_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingStorageImageUpdateAfterBind);
        }

        if self.vk12.descriptor_binding_storage_buffer_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingStorageBufferUpdateAfterBind);
        }

        if self.vk12.descriptor_binding_uniform_texel_buffer_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingUniformTexelBufferUpdateAfterBind);
        }

        if self.vk12.descriptor_binding_storage_texel_buffer_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingStorageTexelBufferUpdateAfterBind);
        }

        if self.vk12.descriptor_binding_update_unused_while_pending == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingUpdateUnusedWhilePending);
        }

        if self.vk12.descriptor_binding_partially_bound == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingPartiallyBound);
        }

        if self.vk12.descriptor_binding_variable_descriptor_count == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingVariableDescriptorCount);
        }

        if self.vk12.runtime_descriptor_array == vk::TRUE {
            set.insert(DeviceFeature::RuntimeDescriptorArray);
        }

        if self.vk12.sampler_filter_minmax == vk::TRUE {
            set.insert(DeviceFeature::SamplerFilterMinmax);
        }

        if self.vk12.scalar_block_layout == vk::TRUE {
            set.insert(DeviceFeature::ScalarBlockLayout);
        }

        if self.vk12.imageless_framebuffer == vk::TRUE {
            set.insert(DeviceFeature::ImagelessFramebuffer);
        }

        if self.vk12.uniform_buffer_standard_layout == vk::TRUE {
            set.insert(DeviceFeature::UniformBufferStandardLayout);
        }

        if self.vk12.shader_subgroup_extended_types == vk::TRUE {
            set.insert(DeviceFeature::ShaderSubgroupExtendedTypes);
        }

        if self.vk12.separate_depth_stencil_layouts == vk::TRUE {
            set.insert(DeviceFeature::SeparateDepthStencilLayouts);
        }

        if self.vk12.host_query_reset == vk::TRUE {
            set.insert(DeviceFeature::HostQueryReset);
        }

        if self.vk12.timeline_semaphore == vk::TRUE {
            set.insert(DeviceFeature::TimelineSemaphore);
        }

        if self.vk12.buffer_device_address == vk::TRUE {
            set.insert(DeviceFeature::BufferDeviceAddress);
        }

        if self.vk12.buffer_device_address_capture_replay == vk::TRUE {
            set.insert(DeviceFeature::BufferDeviceAddressCaptureReplay);
        }

        if self.vk12.buffer_device_address_multi_device == vk::TRUE {
            set.insert(DeviceFeature::BufferDeviceAddressMultiDevice);
        }

        if self.vk12.vulkan_memory_model == vk::TRUE {
            set.insert(DeviceFeature::VulkanMemoryModel);
        }

        if self.vk12.vulkan_memory_model_device_scope == vk::TRUE {
            set.insert(DeviceFeature::VulkanMemoryModelDeviceScope);
        }

        if self.vk12.vulkan_memory_model_availability_visibility_chains == vk::TRUE {
            set.insert(DeviceFeature::VulkanMemoryModelAvailabilityVisibilityChains);
        }

        if self.vk12.shader_output_viewport_index == vk::TRUE {
            set.insert(DeviceFeature::ShaderOutputViewportIndex);
        }

        if self.vk12.shader_output_layer == vk::TRUE {
            set.insert(DeviceFeature::ShaderOutputLayer);
        }

        if self.vk12.subgroup_broadcast_dynamic_id == vk::TRUE {
            set.insert(DeviceFeature::SubgroupBroadcastDynamicId);
        }

        if self.vk13.robust_image_access == vk::TRUE {
            set.insert(DeviceFeature::RobustImageAccess);
        }

        if self.vk13.inline_uniform_block == vk::TRUE {
            set.insert(DeviceFeature::InlineUniformBlock);
        }

        if self.vk13.descriptor_binding_inline_uniform_block_update_after_bind == vk::TRUE {
            set.insert(DeviceFeature::DescriptorBindingInlineUniformBlockUpdateAfterBind);
        }

        if self.vk13.pipeline_creation_cache_control == vk::TRUE {
            set.insert(DeviceFeature::PipelineCreationCacheControl);
        }

        if self.vk13.private_data == vk::TRUE {
            set.insert(DeviceFeature::PrivateData);
        }

        if self.vk13.shader_demote_to_helper_invocation == vk::TRUE {
            set.insert(DeviceFeature::ShaderDemoteToHelperInvocation);
        }

        if self.vk13.shader_terminate_invocation == vk::TRUE {
            set.insert(DeviceFeature::ShaderTerminateInvocation);
        }

        if self.vk13.compute_full_subgroups == vk::TRUE {
            set.insert(DeviceFeature::ComputeFullSubgroups);
        }

        if self.vk13.synchronization2 == vk::TRUE {
            set.insert(DeviceFeature::Synchronization2);
        }

        if self.vk13.texture_compression_astc_hdr == vk::TRUE {
            set.insert(DeviceFeature::TextureCompressionASTCHDR);
        }

        if self.vk13.shader_zero_initialize_workgroup_memory == vk::TRUE {
            set.insert(DeviceFeature::ShaderZeroInitializeWorkgroupMemory);
        }

        if self.vk13.dynamic_rendering == vk::TRUE {
            set.insert(DeviceFeature::DynamicRendering);
        }

        if self.vk13.shader_integer_dot_product == vk::TRUE {
            set.insert(DeviceFeature::ShaderIntegerDotProduct);
        }

        if self.vk13.maintenance4 == vk::TRUE {
            set.insert(DeviceFeature::Maintenance4);
        }

        set
    }
}


pub struct QueueRequest {
    pub family: u32,
    pub count: u32,
    pub label: Option<QueueLabel>,
    pub allow_merge: bool,
}

impl QueueRequest {
    pub const fn strict_labeled(family: u32, count: u32, label: QueueLabel) -> Self {
        Self {
            family, count, label: Some(label), allow_merge: false,
        }
    }

    pub const fn strict_labeled_custom(family: u32, count: u32, label: &'static str) -> Self {
        Self {
            family, count, label: Some(QueueLabel::Custom(label)), allow_merge: false,
        }
    }

    pub const fn strict_unlabeled(family: u32, count: u32) -> Self {
        Self {
            family, count, label: None, allow_merge: false,
        }
    }

    pub const fn flexible_labeled(family: u32, count: u32, label: QueueLabel) -> Self {
        Self {
            family, count, label: Some(label), allow_merge: true,
        }
    }

    pub const fn flexible_labeled_custom(family: u32, count: u32, label: &'static str) -> Self {
        Self {
            family, count, label: Some(QueueLabel::Custom(label)), allow_merge: true,
        }
    }

    pub const fn flexible_unlabeled(family: u32, count: u32) -> Self {
        Self {
            family, count, label: None, allow_merge: true,
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct ExtensionRequest {
    pub name: &'static CStr,
    pub required: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DeviceFeature {
    // Vulkan 1.0
    RobustBufferAccess,
    FullDrawIndexUint32,
    ImageCubeArray,
    IndependentBlend,
    GeometryShader,
    TessellationShader,
    SampleRateShading,
    DualSourceBlend,
    LogicOperation,
    MultiDrawIndirect,
    WideLines,
    LargePoints,
    AlphaToOne,
    MultiViewport,
    SamplerAnisotropy,
    TextureCompressionETC2,
    TextureCompressionASTCLDR,
    TextureCompressionBC,
    OcclusionQueryPrecise,
    PipelineStatisticsQuery,
    VertexPipelineStoresAndAtomics,
    FragmentStoresAndAtomics,
    ShaderTessellationAndGeometryPointSize,
    ShaderImageGatherExtended,
    ShaderStorageImageExtendedFormats,
    ShaderStorageImageMultisample,
    ShaderStorageImageReadWithoutFormat,
    ShaderStorageImageWriteWithoutFormat,
    ShaderUniformBufferArrayDynamicIndexing,
    ShaderSampledImageArrayDynamicIndexing,
    ShaderStorageBufferArrayDynamicIndexing,
    ShaderStorageImageArrayDynamicIndexing,
    ShaderClipDistance,
    ShaderCullDistance,
    ShaderFloat64,
    ShaderInt64,
    ShaderInt16,
    ShaderResourceResidency,
    ShaderResourceMinLod,
    SparseBinding,
    SparseResidencyBuffer,
    SparseResidencyImage2D,
    SparseResidencyImage3D,
    SparseResidency2Samples,
    SparseResidency4Samples,
    SparseResidency8Samples,
    SparseResidency16Samples,
    VariableMultisampleRate,
    InheritedQueries,

    // Vulkan 1.1
    StorageBuffer16BitAccess,
    UniformAndStorageBuffer16BitAccess,
    StoragePushConstant16,
    StorageInputOutput16,
    Multiview,
    MultiviewGeometryShader,
    MultiviewTessellationShader,
    VariablePointersStorageBuffer,
    VariablePointers,
    ProtectedMemory,
    SamplerYcbcrConversion,
    ShaderDrawParameters,

    // Vulkan 1.2
    SamplerMirrorClampToEdge,
    DrawIndirectCount,
    StorageBuffer8BitAccess,
    UniformAndStorageBuffer8BitAccess,
    ShaderBufferInt64Atomics,
    ShaderSharedInt64Atomics,
    ShaderFloat16,
    ShaderInt8,
    DescriptorIndexing,
    ShaderInputAttachmentArrayDynamicIndexing,
    ShaderUniformTexelBufferArrayDynamicIndexing,
    ShaderStorageTexelBufferArrayDynamicIndexing,
    ShaderUniformBufferArrayNonUniformIndexing,
    ShaderSampledImageArrayNonUniformIndexing,
    ShaderStorageBufferArrayNonUniformIndexing,
    ShaderStorageImageArrayNonUniformIndexing,
    ShaderInputAttachmentArrayNonUniformIndexing,
    ShaderUniformTexelBufferArrayNonUniformIndexing,
    ShaderStorageTexelBufferArrayNonUniformIndexing,
    DescriptorBindingUniformBufferUpdateAfterBind,
    DescriptorBindingSampledImageUpdateAfterBind,
    DescriptorBindingStorageImageUpdateAfterBind,
    DescriptorBindingStorageBufferUpdateAfterBind,
    DescriptorBindingUniformTexelBufferUpdateAfterBind,
    DescriptorBindingStorageTexelBufferUpdateAfterBind,
    DescriptorBindingUpdateUnusedWhilePending,
    DescriptorBindingPartiallyBound,
    DescriptorBindingVariableDescriptorCount,
    RuntimeDescriptorArray,
    SamplerFilterMinmax,
    ScalarBlockLayout,
    ImagelessFramebuffer,
    UniformBufferStandardLayout,
    ShaderSubgroupExtendedTypes,
    SeparateDepthStencilLayouts,
    HostQueryReset,
    TimelineSemaphore,
    BufferDeviceAddress,
    BufferDeviceAddressCaptureReplay,
    BufferDeviceAddressMultiDevice,
    VulkanMemoryModel,
    VulkanMemoryModelDeviceScope,
    VulkanMemoryModelAvailabilityVisibilityChains,
    ShaderOutputViewportIndex,
    ShaderOutputLayer,
    SubgroupBroadcastDynamicId,

    // Vulkan 1.3
    RobustImageAccess,
    InlineUniformBlock,
    DescriptorBindingInlineUniformBlockUpdateAfterBind,
    PipelineCreationCacheControl,
    PrivateData,
    ShaderDemoteToHelperInvocation,
    ShaderTerminateInvocation,
    ComputeFullSubgroups,
    Synchronization2,
    TextureCompressionASTCHDR,
    ShaderZeroInitializeWorkgroupMemory,
    DynamicRendering,
    ShaderIntegerDotProduct,
    Maintenance4,
}

#[derive(Clone, Debug, Hash)]
pub struct DeviceFeatureRequest {
    pub feature: DeviceFeature,
    pub required: bool,
}

impl DeviceFeatureRequest {
    pub const fn required(feature: DeviceFeature) -> DeviceFeatureRequest {
        Self {
            feature,
            required: true,
        }
    }

    pub const fn optional(feature: DeviceFeature) -> DeviceFeatureRequest {
        Self {
            feature,
            required: false,
        }
    }
}

impl ExtensionRequest {
    pub const fn required(name: &'static CStr) -> ExtensionRequest {
        Self {
            name,
            required: true,
        }
    }

    pub const fn optional(name: &'static CStr) -> ExtensionRequest {
        Self {
            name,
            required: false,
        }
    }
}

pub trait RequestHelper<R> {
    fn optional(self, value: R) -> Self;
    fn required(self, value: R) -> Self;
}

impl RequestHelper<DeviceFeature> for &mut Vec<DeviceFeatureRequest> {
    fn optional(self, value: DeviceFeature) -> Self {
        self.push(DeviceFeatureRequest::optional(value));
        self
    }

    fn required(self, value: DeviceFeature) -> Self {
        self.push(DeviceFeatureRequest::required(value));
        self
    }
}


impl RequestHelper<&'static CStr> for &mut Vec<ExtensionRequest> {
    fn optional(self, value: &'static CStr) -> Self {
        self.push(ExtensionRequest::optional(value));
        self
    }

    fn required(self, value: &'static CStr) -> Self {
        self.push(ExtensionRequest::required(value));
        self
    }
}