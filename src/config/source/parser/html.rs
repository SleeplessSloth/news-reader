/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod query;

use serde::{Deserialize, Serialize};

use self::query::{IdQuery, ImageQuery, Query, TextQuery, TitleQuery, UrlQuery};
use crate::source;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Html {
	#[serde(rename = "item_query")]
	pub(crate) itemq: Vec<Query>,

	#[serde(rename = "title_query")]
	pub(crate) titleq: Option<TitleQuery>,

	#[serde(rename = "text_query")]
	pub(crate) textq: Vec<TextQuery>,

	#[serde(rename = "id_query")]
	pub(crate) idq: IdQuery,

	#[serde(rename = "link_query")]
	pub(crate) linkq: UrlQuery,

	#[serde(rename = "img_query")]
	pub(crate) imgq: Option<ImageQuery>,
}

impl Html {
	pub(crate) fn parse(self) -> source::parser::Html {
		source::parser::Html {
			itemq: self.itemq.into_iter().map(Query::parse).collect(),
			titleq: self.titleq.map(TitleQuery::parse),
			textq: self.textq.into_iter().map(TextQuery::parse).collect(),
			idq: self.idq.parse(),
			linkq: self.linkq.parse(),
			imgq: self.imgq.map(ImageQuery::parse),
		}
	}
}
