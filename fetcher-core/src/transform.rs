/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod caps;
pub mod html;
pub mod json;
pub mod rss;

pub use self::caps::Caps;
pub use self::html::Html;
pub use self::json::Json;
pub use self::rss::Rss;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::entry::Entry;
use crate::error::transform::Error as TransformError;
use crate::error::transform::Kind as TransformErrorKind;
use crate::read_filter::ReadFilter;
use crate::sink::Message;
use crate::source::with_shared_rf::http::TransformFromField;
use crate::source::Http;

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Rss (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
// TODO: add raw_contents -> msg.body transformator
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Transform {
	// transform from one data to another
	Http,
	Html(Html),
	Json(Json),
	Rss(Rss),

	// filter data
	ReadFilter(Arc<RwLock<ReadFilter>>),

	// modify data in-place
	Caps(Caps),
}

impl Transform {
	/// Transform the entry `entry` into one or more entries
	///
	/// # Errors
	/// if there was an error parsing the entry
	pub async fn transform(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
		Ok(if let Transform::ReadFilter(rf) = self {
			rf.read().await.remove_read_from(&mut entries);
			entries
		} else {
			let mut fully_transformed_entries = Vec::new();
			for entry in entries {
				fully_transformed_entries.extend(self.transform_one(entry).await?);
			}

			fully_transformed_entries
		})
	}

	async fn transform_one(&self, mut entry: Entry) -> Result<Vec<Entry>, TransformError> {
		let res: Result<_, TransformErrorKind> = match self {
			Transform::Http => Http::transform(&entry, TransformFromField::MessageLink)
				.await
				.map_err(Into::into), // TODO: make this a choise
			Transform::Html(x) => x.transform(&entry).map_err(Into::into),
			Transform::Json(x) => x.transform(&entry).map_err(Into::into),
			Transform::Rss(x) => x.transform(&entry).map_err(Into::into),
			// Transform::ReadFilter(rf) => Ok(rf.read().await.transform(&entries)),
			Transform::ReadFilter(_) => {
				unreachable!("Read filter doesn't support transforming one by one")
			}
			Transform::Caps(x) => Ok(x.transform(&entry)),
		};

		res.map_err(|kind| TransformError {
			kind,
			original_entry: entry.clone(),
		})
		.map(|v| {
			v.into_iter()
				// use old entry's value if some new entry's field is None
				.map(|new_entry| Entry {
					id: new_entry.id.or_else(|| entry.id.take()),
					raw_contents: new_entry.raw_contents.or_else(|| entry.raw_contents.take()),
					msg: Message {
						title: new_entry.msg.title.or_else(|| entry.msg.title.take()),
						body: new_entry.msg.body.or_else(|| entry.msg.body.take()),
						link: new_entry.msg.link.or_else(|| entry.msg.link.take()),
						media: new_entry.msg.media.or_else(|| entry.msg.media.take()),
					},
				})
				.collect()
		})
	}
}
