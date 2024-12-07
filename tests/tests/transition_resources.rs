use wgpu::{hal::TextureUses, TextureTransition};
use wgpu_test::{gpu_test, GpuTestConfiguration};
use wgt::{
    CommandEncoderDescriptor, Extent3d, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};

#[gpu_test]
static TRANSITION_RESOURCES: GpuTestConfiguration = GpuTestConfiguration::new().run_sync(|ctx| {
    let texture = ctx.device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&CommandEncoderDescriptor { label: None });

    encoder.transition_resources(
        &[],
        &[TextureTransition {
            texture: &texture,
            selector: None,
            state: TextureUses::COLOR_TARGET,
        }],
    );

    ctx.queue.submit([encoder.finish()]);
});
