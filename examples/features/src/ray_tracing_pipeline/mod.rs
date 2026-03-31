use glam::{Mat4, Vec3};
use std::mem;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{include_wgsl, BufferUsages, IndexFormat, SamplerDescriptor};
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureUpdateMode, BlasBuildEntry, BlasGeometries,
    BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor,
    CreateBlasDescriptor, CreateTlasDescriptor, Tlas, TlasInstance,
};

use crate::utils;

/// Ray tracing pipeline example.
///
/// This example demonstrates the ray tracing pipeline API by tracing rays through
/// a scene with a single triangle, using dedicated ray generation, miss, and
/// closest-hit shader stages instead of inline ray queries.
///
/// NOTE: This example requires `Features::EXPERIMENTAL_RAY_TRACING_PIPELINE` which
/// is currently only supported on Vulkan. The naga SPIR-V backend does not yet support
/// emitting ray tracing pipeline shader stages, so this example will not run end-to-end
/// until that support is added (or shaders are provided via SPIR-V passthrough).
struct Example {
    tlas: Tlas,
    rt_pipeline: wgpu::RayTracingPipeline,
    blit_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    blit_bind_group: wgpu::BindGroup,
    storage_texture: wgpu::Texture,
    animation_timer: utils::AnimationTimer,
}

impl crate::framework::Example for Example {
    fn required_features() -> wgpu::Features {
        wgpu::Features::EXPERIMENTAL_RAY_TRACING_PIPELINE | wgpu::Features::EXPERIMENTAL_RAY_QUERY
    }

    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
            .using_minimum_supported_acceleration_structure_values()
            .using_minimum_supported_ray_tracing_pipeline_values()
    }

    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities::default()
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let blit_shader =
            device.create_shader_module(include_wgsl!("../ray_traced_triangle/blit.wgsl"));

        // -- Acceleration structure setup (same as ray_traced_triangle) --

        let vertices: [f32; 9] = [1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 0.0, -1.0, 0.0];
        let indices: [u32; 3] = [0, 1, 2];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::BLAS_INPUT,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::BLAS_INPUT,
        });

        let blas_size_desc = BlasTriangleGeometrySizeDescriptor {
            vertex_format: wgpu::VertexFormat::Float32x3,
            vertex_count: (vertices.len() / 3) as u32,
            index_format: Some(IndexFormat::Uint32),
            index_count: Some(indices.len() as u32),
            flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
        };

        let blas = device.create_blas(
            &CreateBlasDescriptor {
                label: None,
                flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: AccelerationStructureUpdateMode::Build,
            },
            BlasGeometrySizeDescriptors::Triangles {
                descriptors: vec![blas_size_desc.clone()],
            },
        );

        let mut tlas = device.create_tlas(&CreateTlasDescriptor {
            label: None,
            max_instances: 3,
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::Build,
        });

        tlas[0] = Some(TlasInstance::new(
            &blas,
            Mat4::IDENTITY.transpose().to_cols_array()[..12]
                .try_into()
                .unwrap(),
            0,
            0xff,
        ));

        tlas[1] = Some(TlasInstance::new(
            &blas,
            Mat4::from_translation(Vec3::new(-1.0, -1.0, -2.0))
                .transpose()
                .to_cols_array()[..12]
                .try_into()
                .unwrap(),
            0,
            0xff,
        ));

        tlas[2] = Some(TlasInstance::new(
            &blas,
            Mat4::from_translation(Vec3::new(1.0, -1.0, -2.0))
                .transpose()
                .to_cols_array()[..12]
                .try_into()
                .unwrap(),
            0,
            0xff,
        ));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        encoder.build_acceleration_structures(
            Some(&BlasBuildEntry {
                blas: &blas,
                geometry: BlasGeometries::TriangleGeometries(vec![BlasTriangleGeometry {
                    size: &blas_size_desc,
                    vertex_buffer: &vertex_buffer,
                    first_vertex: 0,
                    vertex_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    index_buffer: Some(&index_buffer),
                    first_index: Some(0),
                    transform_buffer: None,
                    transform_buffer_offset: None,
                }]),
            }),
            Some(&tlas),
        );

        queue.submit(Some(encoder.finish()));

        // -- Storage texture for RT output --

        let storage_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // -- Bind group layouts --

        let rt_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rt bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::RAY_GENERATION
                        | wgpu::ShaderStages::CLOSEST_HIT
                        | wgpu::ShaderStages::MISS,
                    ty: wgpu::BindingType::AccelerationStructure {
                        vertex_return: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::RAY_GENERATION,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let blit_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        // -- Pipelines --

        let rt_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt pipeline layout"),
            bind_group_layouts: &[Some(&rt_bgl)],
            immediate_size: 0,
        });

        let rt_pipeline = device.create_ray_tracing_pipeline(&wgpu::RayTracingPipelineDescriptor {
            label: Some("rt pipeline"),
            layout: Some(&rt_pipeline_layout),
            ray_generation: wgpu::RayTracingShaderStage {
                module: &shader,
                entry_point: Some("raygen"),
                compilation_options: Default::default(),
            },
            miss: &[wgpu::RayTracingShaderStage {
                module: &shader,
                entry_point: Some("miss_main"),
                compilation_options: Default::default(),
            }],
            hit_groups: &[wgpu::RayTracingHitGroup {
                closest_hit: wgpu::RayTracingShaderStage {
                    module: &shader,
                    entry_point: Some("closest_hit_main"),
                    compilation_options: Default::default(),
                },
                any_hit: None,
            }],
            max_recursion_depth: 1,
            cache: None,
        });

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blit pipeline layout"),
            bind_group_layouts: &[Some(&blit_bgl)],
            immediate_size: 0,
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        // -- Bind groups --

        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rt bind group"),
            layout: &rt_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::AccelerationStructure(&tlas),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &storage_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blit bind group"),
            layout: &blit_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &storage_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            tlas,
            rt_pipeline,
            blit_pipeline,
            bind_group,
            blit_bind_group,
            storage_texture: storage_tex,
            animation_timer: utils::AnimationTimer::default(),
        }
    }

    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {}

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.tlas[0].as_mut().unwrap().transform =
            Mat4::from_rotation_y(self.animation_timer.time())
                .transpose()
                .to_cols_array()[..12]
                .try_into()
                .unwrap();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.build_acceleration_structures(None, Some(&self.tlas));

        // Ray tracing pass
        {
            let mut rtpass = encoder.begin_ray_tracing_pass(&wgpu::RayTracingPassDescriptor {
                label: Some("rt pass"),
                timestamp_writes: None,
            });
            rtpass.set_pipeline(&self.rt_pipeline);
            rtpass.set_bind_group(0, Some(&self.bind_group), &[]);
            rtpass.trace_rays(
                self.storage_texture.width(),
                self.storage_texture.height(),
                1,
            );
        }

        // Blit to screen
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_pipeline(&self.blit_pipeline);
            rpass.set_bind_group(0, Some(&self.blit_bind_group), &[]);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

pub fn main() {
    crate::framework::run::<Example>("ray-tracing-pipeline");
}
