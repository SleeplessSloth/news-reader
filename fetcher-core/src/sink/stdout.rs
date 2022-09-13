/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::sink::Error as SinkError;
use crate::sink::Message;

use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct Stdout;

impl Stdout {
	/// Send a message with a tag to stdout
	///
	/// # Errors
	/// if there was an error writing to stdout
	pub async fn send(&self, msg: Message, tag: Option<&str>) -> Result<(), SinkError> {
		tokio::io::stdout().write_all(format!(
			"------------------------------\nMessage:\nTitle: {title:?}\n\nBody:\n{body}\n\nLink: {link:?}\n\nMedia: {media:?}\n\nTag: {tag:?}\n------------------------------\n",
			title = msg.title.as_deref(),
			body = msg.body.as_deref().unwrap_or("None"),
			link = msg.link.map(|url| url.as_str().to_owned()).as_deref(),
			media = msg.media,
			tag = tag
		).as_bytes()).await.map_err(SinkError::StdoutWrite)
	}
}
