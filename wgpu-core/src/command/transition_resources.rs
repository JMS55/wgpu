use crate::{global::Global, id::CommandEncoderId};

use super::CommandEncoderError;

impl Global {
    pub fn command_encoder_transition_resources(
        &self,
        command_encoder_id: CommandEncoderId,
        buffer_transitions: &[()],
        texture_transitions: &[()],
    ) -> Result<(), CommandEncoderError> {
        profiling::scope!("CommandEncoder::transition_resources");

        Ok(())
    }
}
