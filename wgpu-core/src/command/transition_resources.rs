use hal::{BufferUses, TextureUses};
use thiserror::Error;

use crate::{
    command::CommandBuffer,
    device::DeviceError,
    global::Global,
    id::{BufferId, CommandEncoderId, TextureId},
    resource::{InvalidResourceError, ParentDevice},
    track::{ResourceUsageCompatibilityError, TextureSelector},
};

use super::CommandEncoderError;

impl Global {
    pub fn command_encoder_transition_resources(
        &self,
        command_encoder_id: CommandEncoderId,
        buffer_transitions: &[(BufferId, BufferUses)],
        texture_transitions: &[(TextureId, Option<TextureSelector>, TextureUses)],
    ) -> Result<(), TransitionResourcesError> {
        profiling::scope!("CommandEncoder::transition_resources");

        let hub = &self.hub;

        // Lock command encoder for recording
        let cmd_buf = hub
            .command_buffers
            .get(command_encoder_id.into_command_buffer_id());
        let mut cmd_buf_data = cmd_buf.data.lock();
        let mut cmd_buf_data_guard = cmd_buf_data.record()?;
        let cmd_buf_data = &mut *cmd_buf_data_guard;

        // Get and lock device
        let device = &cmd_buf.device;
        device.check_is_valid()?;
        let snatch_guard = &device.snatchable_lock.read();

        let mut usage_scope = device.new_usage_scope();

        // Process buffer transitions
        for (buffer_id, state) in buffer_transitions {
            let buffer = hub.buffers.get(*buffer_id).get()?;
            buffer.same_device_as(cmd_buf.as_ref())?;

            usage_scope.buffers.merge_single(&buffer, *state)?;
        }

        // Process texture transitions

        for (texture_id, selector, state) in texture_transitions {
            let texture = hub.textures.get(*texture_id).get()?;
            texture.same_device_as(cmd_buf.as_ref())?;

            unsafe {
                usage_scope
                    .textures
                    .merge_single(&texture, selector.clone(), *state)
            }?;
        }

        // Record any needed barriers based on tracker data
        let cmd_buf_raw = cmd_buf_data.encoder.open(device)?;
        CommandBuffer::insert_barriers_from_scope(
            cmd_buf_raw,
            &mut cmd_buf_data.trackers,
            &usage_scope,
            snatch_guard,
        );
        cmd_buf_data_guard.mark_successful();

        Ok(())
    }
}

/// Error encountered while attempting to perform [`Global::command_encoder_transition_resources`].
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum TransitionResourcesError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error(transparent)]
    Encoder(#[from] CommandEncoderError),
    #[error(transparent)]
    InvalidResource(#[from] InvalidResourceError),
    #[error(transparent)]
    ResourceUsage(#[from] ResourceUsageCompatibilityError),
}
