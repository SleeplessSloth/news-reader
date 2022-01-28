use std::time::Duration;
use teloxide::{
	adaptors::{throttle::Limits, Throttle},
	payloads::SendMessageSetters,
	requests::{Request, Requester, RequesterExt},
	types::{
		ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, Message as TelMessage,
		ParseMode,
	},
	Bot, RequestError,
};

use crate::error::{Error, Result};

pub enum Media {
	Photo(String),
	Video(String),
}

pub struct Message {
	pub text: String,
	pub media: Option<Vec<Media>>,
}

pub struct Telegram {
	bot: Throttle<Bot>,
	chat_id: ChatId,
}


impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message").field("text", &self.text).field("media.is_some()", &self.media.is_some()).finish()
    }
}

impl Telegram {
	pub fn new(bot: Bot, chat_id: impl Into<ChatId>) -> Self {
		Self {
			bot: bot.throttle(Limits::default()),
			chat_id: chat_id.into(),
		}
	}

	#[tracing::instrument]
	pub async fn send(&self, message: Message) -> Result<()> {
		// NOTE: workaround for some kind of a bug that doesn't let access both text and media fields of the struct in the map closure at once
		let text = if message.text.len() > 4096 {
			tracing::warn!("Message too long ({len})", len = message.text.len());
			let (idx, _) = message.text.char_indices().nth(4096 - 3).unwrap(); // NOTE: safe unwrap, length already confirmed to be bigger
			let mut m = message.text[..idx].to_string();
			m.push_str("...");
			m
		} else {
			message.text
		};

		if let Some(media) = message.media {
			self.send_media(
				media
					.into_iter()
					.map(|x| match x {
						Media::Photo(url) => InputMedia::Photo(
							InputMediaPhoto::new(InputFile::url(url))
								.caption(text.clone())
								.parse_mode(ParseMode::Html),
						),
						Media::Video(url) => InputMedia::Video(
							InputMediaVideo::new(InputFile::url(url))
								.caption(text.clone())
								.parse_mode(ParseMode::Html),
						),
					})
					.collect::<Vec<InputMedia>>(),
			)
			.await?;
		} else {
			self.send_text(text).await?;
		}

		Ok(())
	}

	#[tracing::instrument]
	async fn send_text(&self, message: String) -> Result<TelMessage> {
		loop {
			tracing::info!("Sending text message");
			match self
				.bot
				.send_message(self.chat_id.clone(), &message)
				.parse_mode(ParseMode::Html)
				.disable_web_page_preview(true)
				.send()
				.await
			{
				Ok(message) => return Ok(message),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::warn!("Exceeded rate limit, retrying in {retry_after}");
					tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
				}
				Err(e) => return Err(Error::Send { why: e.to_string() }),
			}
		}
	}

	#[tracing::instrument]
	async fn send_media(&self, media: Vec<InputMedia>) -> Result<Vec<TelMessage>> {
		loop {
			tracing::info!("Sending media message");
			match self
				.bot
				.send_media_group(self.chat_id.clone(), media.clone())
				.send()
				.await
			{
				Ok(messages) => return Ok(messages),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::warn!("Exceeded rate limit, retrying in {retry_after}");
					tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
				}
				Err(e) => return Err(Error::Send { why: e.to_string() }),
			}
		}
	}
}

impl std::fmt::Debug for Telegram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Telegram")
			.field("chat_id", &self.chat_id)
			.finish_non_exhaustive()
	}
}