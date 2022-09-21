/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::auth::Google as CoreGoogleAuth;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Google {
	client_id: String,
	client_secret: String,
	refresh_token: String,
}

impl Google {
	pub fn parse(self) -> CoreGoogleAuth {
		CoreGoogleAuth::new(self.client_id, self.client_secret, self.refresh_token)
	}

	pub fn unparse(auth: CoreGoogleAuth) -> Self {
		let CoreGoogleAuth {
			client_id,
			client_secret,
			refresh_token,
			..
		} = auth;

		Self {
			client_id,
			client_secret,
			refresh_token,
		}
	}
}