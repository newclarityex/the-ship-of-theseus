use ringbuf::HeapProducer;

use crate::{
	tween::{Tween, Value},
	CommandError, Volume,
};

use super::Command;

/// Controls a delay effect.
pub struct DelayHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl DelayHandle {
	/// Sets the delay time (in seconds).
	pub fn set_delay_time(
		&mut self,
		delay_time: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetDelayTime(delay_time.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the amount of feedback.
	pub fn set_feedback(
		&mut self,
		feedback: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFeedback(feedback.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn set_mix(
		&mut self,
		mix: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
