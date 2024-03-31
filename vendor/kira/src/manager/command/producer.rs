use crate::error::CommandError;
use ringbuf::HeapProducer;
use std::sync::{Arc, Mutex};

use super::Command;

#[derive(Clone)]
pub(crate) struct CommandProducer(Arc<Mutex<HeapProducer<Command>>>);

impl CommandProducer {
	pub fn new(raw_producer: HeapProducer<Command>) -> Self {
		Self(Arc::new(Mutex::new(raw_producer)))
	}

	pub fn push(&self, command: Command) -> Result<(), CommandError> {
		self.0
			.lock()
			.map_err(|_| CommandError::MutexPoisoned)?
			.push(command)
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
