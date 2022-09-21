/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Twitter feed
//!
//! This module includes the [`Twitter`] struct that is a source that is able to parse a twitter feed via twitter API

use crate::entry::Entry;
use crate::error::source::TwitterError;
use crate::sink::Media;
use crate::sink::Message;

use egg_mode::entities::MediaType;
use egg_mode::{auth::bearer_token, tweet::user_timeline, KeyPair, Token};

/// A source that fetches from a Twitter feed using the Twitter API
pub struct Twitter {
	handle: String,
	api_key: String,
	api_secret: String,
	token: Option<Token>,
	filter: Vec<String>,
}

impl Twitter {
	/// Creates a new [`Twitter`] source
	#[must_use]
	pub fn new(handle: String, api_key: String, api_secret: String, filter: Vec<String>) -> Self {
		Self {
			handle,
			api_key,
			api_secret,
			token: None,
			filter,
		}
	}

	/// Fetches all tweets from the feed
	#[tracing::instrument(skip_all)]
	pub async fn get(&mut self) -> Result<Vec<Entry>, TwitterError> {
		tracing::debug!("Getting tweets");

		let token = match &self.token {
			Some(t) => t,
			None => {
				self.token = Some(
					bearer_token(&KeyPair::new(self.api_key.clone(), self.api_secret.clone()))
						.await
						.map_err(TwitterError::Auth)?,
				);

				self.token
					.as_ref()
					.expect("token should have been init just up above")
			}
		};

		// TODO: keep a tweet id -> message id hashmap and handle enable with_replies from below
		let (_, tweets) = user_timeline(self.handle.clone(), false, true, token)
			/*
			// TODO: is this doing what I think it is doing or have I gotten it wrong? The docs aren't clear enough
			.older(
				// read_filter
				// 	.and_then(ReadFilter::last_read)
				// 	.and_then(|x| x.parse().ok()),
				if let Some(rf) = &self.read_filter {
					if let Some(last_read_id) = rf.read().await.last_read() {
						last_read_id.parse().ok()
					} else {
						None
					}
				} else {
					None
				},
			)
			*/
			.start()
			.await?;

		tracing::debug!(
			"Got {num} tweets older than the last one read",
			num = tweets.len()
		);

		let messages = tweets
			.iter()
			.filter_map(|tweet| {
				if !self.filter.is_empty()
					&& !Self::tweet_contains_filters(&tweet.text, self.filter.as_slice())
				{
					return None;
				}

				Some(Entry {
					id: Some(tweet.id.to_string()),
					msg: Message {
						body: Some(tweet.text.clone()),
						link: Some(
							format!(
								"https://twitter.com/{handle}/status/{id}",
								handle = self.handle,
								id = tweet.id
							)
							.as_str()
							.try_into()
							.expect("The URL is hand crafted and should always be valid"),
						),
						media: tweet.entities.media.as_ref().and_then(|x| {
							x.iter()
								.map(|x| match x.media_type {
									MediaType::Photo => {
										Some(Media::Photo(x.media_url.as_str().try_into().expect("The tweet URL provided by the Tweeter API should always be a valid URL")))
									}
									MediaType::Video => {
										Some(Media::Video(x.media_url.as_str().try_into().expect("The tweet URL provided by the Tweeter API should always be a valid URL")))
									}
									MediaType::Gif => None,
								})
								.collect::<Option<Vec<Media>>>()
						}),
						..Default::default()
					},
					..Default::default()
				})
			})
			.collect::<Vec<_>>();

		let unread_num = messages.len();
		if unread_num > 0 {
			tracing::debug!("Got {unread_num} unread filtered tweets");
		} else {
			tracing::debug!("All tweets have already been read, none remaining to send");
		}

		Ok(messages)
	}

	fn tweet_contains_filters(tweet: &str, filters: &[String]) -> bool {
		for filter in filters {
			if !tweet.to_lowercase().contains(&filter.to_lowercase()) {
				return false;
			}
		}

		true
	}
}

impl std::fmt::Debug for Twitter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Twitter")
			.field("handle", &self.handle)
			.field("filter", &self.filter)
			.finish_non_exhaustive()
	}
}