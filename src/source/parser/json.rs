/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde_json::Value;
use url::Url;

use crate::entry::Entry;
use crate::error::{Error, Result};
use crate::sink::{Media, Message};

#[derive(Debug)]
pub(crate) struct TextQuery {
	pub(crate) string: String,
	pub(crate) prepend: Option<String>,
	pub(crate) append: Option<String>,
}

// TODO: differantiate between nested and adjecent fields more clearly, here and in HTML parser, too
#[derive(Debug)]
pub struct Json {
	pub(crate) itemq: Vec<String>,
	pub(crate) titleq: Option<String>,
	pub(crate) textq: Vec<TextQuery>, // adjecent
	pub(crate) idq: String,
	pub(crate) linkq: Option<TextQuery>,
	pub(crate) imgq: Option<Vec<String>>, // nested
}

impl Json {
	#[tracing::instrument(skip_all)]
	pub fn parse(&self, entry: Entry) -> Result<Vec<Entry>> {
		let json: Value = serde_json::from_str(&entry.msg.body)?;

		let items = self.itemq.iter().try_fold(&json, |acc, x| {
			acc.get(x.as_str())
				.ok_or_else(|| Error::JsonParseKeyNotFound(x.clone()))
		})?;

		let items_iter: Box<dyn Iterator<Item = &Value>> = if let Some(items) = items.as_array() {
			Box::new(items.iter())
		} else if let Some(items) = items.as_object() {
			// ignore map keys, iterate over values only
			Box::new(items.iter().map(|(_, v)| v))
		} else {
			return Err(Error::JsonParseKeyWrongType {
				key: self.itemq.last().unwrap().clone(),
				expected_type: "iterator (array, map)",
				found_type: format!("{items:?}"),
			});
		};

		items_iter
			.into_iter()
			.map(|item| {
				let title = self
					.titleq
					.as_ref()
					.and_then(|s| item.get(s))
					.and_then(serde_json::Value::as_str)
					.map(|s| s.trim().to_owned());

				let text = self
					.textq
					.iter()
					.map(|query| {
						let mut text_str = {
							let text_val = item
								.get(&query.string)
								.ok_or_else(|| Error::JsonParseKeyNotFound(query.string.clone()))?;

							text_val
								.as_str()
								.ok_or_else(|| Error::JsonParseKeyWrongType {
									key: query.string.clone(),
									expected_type: "string",
									found_type: format!("{text_val:?}"),
								})?
								.trim()
								.to_owned()
						};

						if query.prepend.is_some() || query.append.is_some() {
							text_str = format!(
								"{prepend}{original}{append}",
								prepend = query.prepend.as_deref().unwrap_or_default(),
								original = text_str,
								append = query.append.as_deref().unwrap_or_default()
							);
						}

						Ok(text_str)
					})
					.collect::<Result<Vec<String>>>()?
					.join("\n\n");

				let id = {
					let id_val = item
						.get(&self.idq)
						.ok_or_else(|| Error::JsonParseKeyNotFound(self.idq.clone()))?;

					if let Some(id) = id_val.as_str() {
						id.to_owned()
					} else if let Some(id) = id_val.as_i64() {
						id.to_string()
					} else if let Some(id) = id_val.as_u64() {
						id.to_string()
					} else {
						return Err(Error::JsonParseKeyWrongType {
							key: self.idq.clone(),
							expected_type: "string/i64/u64",
							found_type: format!("{id_val:?}"),
						});
					}
				};

				let link = self
					.linkq
					.as_ref()
					.map(|linkq| {
						let link_val = item
							.get(&linkq.string)
							.ok_or_else(|| Error::JsonParseKeyNotFound(linkq.string.clone()))?;
						let mut link_str = link_val
							.as_str()
							.ok_or_else(|| Error::JsonParseKeyWrongType {
								key: linkq.string.clone(),
								expected_type: "string",
								found_type: format!("{link_val:?}"),
							})?
							.to_owned();

						if linkq.prepend.is_some() || linkq.append.is_some() {
							link_str = format!(
								"{prepend}{original}{append}",
								prepend = linkq.prepend.as_deref().unwrap_or_default(),
								original = link_str,
								append = linkq.append.as_deref().unwrap_or_default()
							);
						}

						Url::try_from(link_str.as_str())
							.map_err(|e| Error::UrlInvalid(e, link_str.clone()))
					})
					.transpose()?;

				let img: Option<Url> = self
					.imgq
					.as_ref()
					.map(|imgq| {
						let first = item
							.get(&imgq[0])
							.ok_or_else(|| Error::JsonParseKeyNotFound(imgq[0].clone()))?;

						let img_val = imgq.iter().skip(1).try_fold(first, |val, x| {
							val.get(x)
								.ok_or_else(|| Error::JsonParseKeyNotFound(x.clone()))
						})?;

						let img_str = img_val
							.as_str()
							.ok_or_else(|| Error::JsonParseKeyWrongType {
								key: imgq.last().unwrap().clone(),
								expected_type: "string",
								found_type: format!("{img_val:?}"),
							})?
							.to_owned();

						Url::try_from(img_str.as_str())
							.map_err(|e| Error::UrlInvalid(e, img_str.clone()))
					})
					.transpose()?;

				Ok(Entry {
					id,
					msg: Message {
						title,
						body: text,
						link,
						media: img.map(|url| vec![Media::Photo(url)]),
					},
				})
			})
			.collect::<Result<Vec<Entry>>>()
	}
}