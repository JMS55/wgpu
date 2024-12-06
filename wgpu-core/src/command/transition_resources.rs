use thiserror::Error;

use crate::{
    command::CommandBuffer, device::DeviceError, global::Global, id::CommandEncoderId,
    track::ResourceUsageCompatibilityError,
};

use super::CommandEncoderError;

impl Global {
    pub fn command_encoder_transition_resources(
        &self,
        command_encoder_id: CommandEncoderId,
        buffer_transitions: &[()],
        texture_transitions: &[()],
    ) -> Result<(), TransitionResourcesError> {
        profiling::scope!("CommandEncoder::transition_resources");

        let hub = &self.hub;

        let cmd_buf = hub
            .command_buffers
            .get(command_encoder_id.into_command_buffer_id());
        let mut cmd_buf_data = cmd_buf.data.lock();
        let mut cmd_buf_data_guard = cmd_buf_data.record()?;
        let cmd_buf_data = &mut *cmd_buf_data_guard;

        let device = &cmd_buf.device;
        let snatch_guard = &device.snatchable_lock.read();

        let mut usage_scope = device.new_usage_scope();

        for buffer_transition in buffer_transitions {
            usage_scope.buffers.merge_single(todo!(), todo!())?;
        }

        for texture_transition in texture_transitions {
            unsafe { usage_scope.textures.merge_single(todo!(), todo!(), todo!()) }?;
        }

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
    ResourceUsage(#[from] ResourceUsageCompatibilityError),
}
