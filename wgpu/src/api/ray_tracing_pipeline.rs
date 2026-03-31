use crate::*;

/// Handle to a ray tracing pipeline.
///
/// A `RayTracingPipeline` object represents a ray tracing pipeline with dedicated
/// shader stages (ray generation, miss, closest hit, any hit) that can be dispatched
/// via [`RayTracingPass::trace_rays`].
///
/// It can be created with [`Device::create_ray_tracing_pipeline`].
///
/// Requires [`Features::EXPERIMENTAL_RAY_TRACING_PIPELINE`].
#[derive(Debug, Clone)]
pub struct RayTracingPipeline {
    pub(crate) inner: dispatch::DispatchRayTracingPipeline,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(RayTracingPipeline: Send, Sync);

crate::cmp::impl_eq_ord_hash_proxy!(RayTracingPipeline => .inner);

impl RayTracingPipeline {
    /// Get an object representing the bind group layout at a given index.
    ///
    /// If this pipeline was created with a [default layout][RayTracingPipelineDescriptor::layout],
    /// then bind groups created with the returned `BindGroupLayout` can only be used with this
    /// pipeline.
    ///
    /// This method will raise a validation error if there is no bind group layout at `index`.
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        let bind_group = self.inner.get_bind_group_layout(index);
        BindGroupLayout { inner: bind_group }
    }

    #[cfg(custom)]
    /// Returns custom implementation of RayTracingPipeline (if custom backend and is internally T)
    pub fn as_custom<T: custom::RayTracingPipelineInterface>(&self) -> Option<&T> {
        self.inner.as_custom()
    }
}

/// Describes a shader stage in a ray tracing pipeline.
#[derive(Clone, Debug)]
pub struct RayTracingShaderStage<'a> {
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader to use.
    ///
    /// If [`Some`], there must be an entry point with the appropriate stage attribute
    /// (e.g. `@ray_generation`, `@miss`, `@closest_hit`, `@any_hit`) with this name in `module`.
    /// Otherwise, expects exactly one entry point of the appropriate stage.
    pub entry_point: Option<&'a str>,
    /// Advanced options for when this stage is compiled.
    pub compilation_options: PipelineCompilationOptions<'a>,
}

/// Describes a hit group in a ray tracing pipeline.
///
/// A hit group combines a closest-hit shader with an optional any-hit shader.
/// When a ray intersects geometry, the closest-hit shader of the matched hit group
/// is invoked. If an any-hit shader is present, it is invoked for each potential
/// intersection before the closest hit is determined.
#[derive(Clone, Debug)]
pub struct RayTracingHitGroup<'a> {
    /// The closest-hit shader, invoked when a ray finds its closest intersection.
    pub closest_hit: RayTracingShaderStage<'a>,
    /// An optional any-hit shader, invoked for each potential intersection.
    pub any_hit: Option<RayTracingShaderStage<'a>>,
}

/// Describes a ray tracing pipeline.
///
/// For use with [`Device::create_ray_tracing_pipeline`].
///
/// Requires [`Features::EXPERIMENTAL_RAY_TRACING_PIPELINE`].
#[derive(Clone, Debug)]
pub struct RayTracingPipelineDescriptor<'a> {
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    ///
    /// If this is set, then [`Device::create_ray_tracing_pipeline`] will raise a validation error
    /// if the layout doesn't match what the shader module(s) expect.
    ///
    /// If `layout` is `None`, then the pipeline has a default layout created and used instead.
    /// The default layout is deduced from the shader modules.
    pub layout: Option<&'a PipelineLayout>,
    /// The ray generation shader entry point. Exactly one is required.
    ///
    /// This shader is the entry point for ray tracing work, responsible for generating
    /// initial rays via `traceRay`.
    pub ray_generation: RayTracingShaderStage<'a>,
    /// Miss shaders, invoked when a ray does not hit any geometry.
    ///
    /// At least one miss shader is typically needed for a functional ray tracing pipeline.
    pub miss: &'a [RayTracingShaderStage<'a>],
    /// Hit groups, each containing a closest-hit and optional any-hit shader.
    ///
    /// When a ray intersects geometry, the hit group associated with that geometry
    /// determines which shaders are invoked.
    pub hit_groups: &'a [RayTracingHitGroup<'a>],
    /// Maximum ray recursion depth.
    ///
    /// Limits how many times `traceRay` can be called recursively from hit or miss shaders.
    /// A value of 1 means only the initial `traceRay` from the ray generation shader is allowed.
    /// Must not exceed the device's max recursion depth limit.
    pub max_recursion_depth: u32,
    /// The pipeline cache to use when creating this pipeline.
    pub cache: Option<&'a PipelineCache>,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(RayTracingPipelineDescriptor<'_>: Send, Sync);
