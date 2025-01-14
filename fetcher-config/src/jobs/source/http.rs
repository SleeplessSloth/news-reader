/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::{Http as CHttp, http::HttpError as CHttpError};

use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use url::Url;

#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Http(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<Request>);

// treat http: url the same as http: {get: url}
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Request {
	Untagged(Url),
	Tagged(TaggedRequest),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TaggedRequest {
	Get(Url),
	Post { url: Url, body: String },
}

impl Http {
	pub fn decode_from_conf(self) -> Result<Vec<CHttp>, CHttpError> {
		self.0
			.into_iter()
			.map(Request::decode_from_conf)
			.collect::<Result<_, CHttpError>>()
	}
}

impl Request {
	pub fn decode_from_conf(self) -> Result<CHttp, CHttpError> {
		match self {
			Self::Untagged(url) | Self::Tagged(TaggedRequest::Get(url)) => CHttp::new_get(url),
			Self::Tagged(TaggedRequest::Post { url, body }) => CHttp::new_post(url, &body),
		}
	}
}
