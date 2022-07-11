/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::path::PathBuf;

use crate::entry::Entry;
use crate::error::{Error, Result};
use crate::sink::Message;

#[derive(Debug)]
pub struct File {
	pub(crate) path: PathBuf,
}

impl File {
	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Vec<Entry>> {
		let text = tokio::fs::read_to_string(&self.path)
			.await
			.map(|s| s.trim().to_owned())
			.map_err(|e| Error::LocalIoRead(e, self.path.clone()))?;

		Ok(vec![Entry {
			id: String::new(),
			msg: Message {
				title: None,
				body: text,
				link: None,
				media: None,
			},
		}])
	}
}
