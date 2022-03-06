/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use rss::Channel;

use crate::error::Result;
use crate::read_filter::Id;
use crate::read_filter::ReadFilterNewer;
use crate::sink::message::Link;
use crate::sink::message::LinkLocation;
use crate::sink::Message;
use crate::source::Responce;

pub struct Rss {
	// name: String,
	// TODO: use url
	url: String,
	http_client: reqwest::Client,
}

impl Rss {
	#[tracing::instrument(name = "Rss::new")]
	pub fn new(/* name: String, */ url: String) -> Self {
		tracing::info!("Creatng an Rss provider");
		Self {
			// name,
			url,
			http_client: reqwest::Client::new(),
		}
	}

	#[tracing::instrument(name = "Rss::get")]
	pub async fn get(&mut self, read_filter: &ReadFilterNewer) -> Result<Vec<Responce>> {
		tracing::debug!("Getting RSS articles");
		let content = self
			.http_client
			.get(&self.url)
			.send()
			.await?
			.bytes()
			.await?;

		let feed = Channel::read_from(&content[..])?;

		tracing::debug!("Got {num} RSS articles total", num = feed.items.len());

		let mut articles = feed.items;
		read_filter.remove_read_from(&mut articles);

		tracing::debug!("{num} unread RSS articles remaning", num = articles.len());

		let messages = articles
			.into_iter()
			.rev()
			.map(|x| {
				Responce {
					id: Some(x.guid.as_ref().unwrap().value.clone()), // unwrap NOTE: same as above
					msg: Message {
						// unwrap NOTE: "safe", these are required fields
						title: Some(x.title.unwrap()),
						body: x.description.unwrap(),
						link: Some(Link {
							url: x.link.unwrap().as_str().try_into().unwrap(),
							loc: LinkLocation::PreferTitle,
						}), // unwrap FIXME: may be an invalid url
						media: None,
					},
				}
			})
			.collect();

		Ok(messages)
	}
}

impl std::fmt::Debug for Rss {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Rss")
			// .field("name", &self.name)
			.field("url", &self.url)
			.finish_non_exhaustive()
	}
}

impl Id for rss::Item {
	fn id(&self) -> &str {
		self.guid().unwrap().value()
	}
}
