/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod entry_to_msg_map;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap::TapOptional;
use tokio::sync::RwLock;

use super::{
	action::Action,
	external_data::{ExternalDataResult, ProvideExternalData},
	read_filter,
	sink::Sink,
	source::Source,
	JobName, Tag, TaskName,
};
use crate::Error;
use fetcher_core::{task::Task as CTask, utils::OptionExt};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	#[serde(rename = "read_filter_type")]
	pub read_filter_kind: Option<read_filter::Kind>,
	pub tag: Option<Tag>,
	pub source: Option<Source>,
	#[serde(rename = "process")]
	pub actions: Option<Vec<Action>>,
	// TODO: several sinks or integrate into actions
	pub sink: Option<Sink>,
	pub entry_to_msg_map_enabled: Option<bool>,
}

impl Task {
	pub fn parse<D>(
		self,
		job: &JobName,
		task_name: Option<&TaskName>,
		external: &D,
	) -> Result<CTask, Error>
	where
		D: ProvideExternalData + ?Sized,
	{
		let rf = match self.read_filter_kind {
			Some(expected_rf_type) => {
				match external.read_filter(job, task_name, expected_rf_type) {
					ExternalDataResult::Ok(rf) => Some(Arc::new(RwLock::new(rf))),
					ExternalDataResult::Unavailable => {
						tracing::warn!("Read filter is unavailable, skipping");
						None
					}
					ExternalDataResult::Err(e) => return Err(e.into()),
				}
			}
			None => None,
		};

		let actions = self.actions.try_map(|x| {
			x.into_iter()
				.filter_map(|act| act.parse(rf.clone()).transpose())
				.collect::<Result<_, _>>()
		})?;

		let entry_to_msg_map = if self
			.entry_to_msg_map_enabled
			.tap_some(|b| {
				if let Some(sink) = &self.sink {
					// TODO: include task name
					tracing::info!(
						"Overriding entry_to_msg_map_enabled for {} from the default {} to {}",
						job,
						sink.has_message_id_support(),
						b
					);
				}
			})
			.unwrap_or_else(|| {
				self.sink
					.as_ref()
					.map_or(false, Sink::has_message_id_support)
			}) {
			match external.entry_to_msg_map(job, task_name) {
				ExternalDataResult::Ok(v) => Some(v),
				ExternalDataResult::Unavailable => {
					tracing::warn!("Entry to message map is unavailable, skipping...");
					None
				}
				ExternalDataResult::Err(e) => return Err(e.into()),
			}
		} else {
			None
		};

		let tag = self.tag.and_then(|tag| match (tag, task_name) {
			(Tag::String(s), _) => Some(s),
			(Tag::UseTaskName, Some(name)) => Some(name.0.clone()),
			(Tag::UseTaskName, None) => {
				tracing::error!("Can't use a task's name as its tag when it has no name");
				None
			}
		});

		Ok(CTask {
			tag,
			source: self.source.map(|x| x.parse(rf, external)).transpose()?,
			actions,
			sink: self.sink.try_map(|x| x.parse(external))?,
			entry_to_msg_map,
		})
	}
}