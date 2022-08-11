/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// Contains [`Message`] and [`Media`]
pub mod message;
pub(crate) mod stdout;
/// Telegram sink
pub mod telegram;

pub use message::{Media, Message};
pub use stdout::Stdout;
pub use telegram::Telegram;

use crate::error::sink::Error as SinkError;

/// All available sinks
#[derive(Debug)]
pub enum Sink {
	/// Telegram sink
	Telegram(Telegram),
	/// stdout sink
	Stdout(Stdout),
	/// null sink that discards any messages
	Null,
}

impl Sink {
	/// Send a message with an optional tag to the sink
	///
	/// # Errors
	/// if there was an error sending the message
	pub async fn send(&self, message: Message, tag: Option<&str>) -> Result<(), SinkError> {
		match self {
			Self::Telegram(t) => t.send(message, tag).await,
			Self::Stdout(s) => s.send(message, tag).await,
			Self::Null => Ok(()),
		}
	}
}