use std::collections::HashMap;

use twilight_model::guild::PartialMember;
use worker::{Request, Response, RouteContext};

use crate::constants::kv;

const EVE_OAUTH_URL: &'static str = "https://login.eveonline.com/v2/oauth/authorize/";

pub async fn login<T>(req: Request, ctx: RouteContext<T>) -> worker::Result<Response> {
	let kv = ctx.kv(kv::LOGINS)?;
	let url = req.url()?;
	let maybe_id = url.query_pairs().find(|(k, _)| k == "id").map(|(_, v)| v);

	match maybe_id {
		Some(id) => {
			let maybe_user_data = kv.get(&id).await?;
			match maybe_user_data {
				Some(_) => Ok(Response::empty()?.with_headers(
					[(
						"Location".to_string(),
						format!(
							"{}?response_type=code&client_id={}&state={}",
							EVE_OAUTH_URL,
							ctx.var("EVE_CLIENT_ID")?.to_string(),
							&id
						),
					)]
					.iter()
					.collect(),
				)),
				None => Response::error("Unknown user", 400),
			}
		}
		None => Response::error("Unknown user", 400),
	}
}

pub async fn callback<T>(req: Request, ctx: RouteContext<T>) -> worker::Result<Response> {
	let url = req.url()?;
	let query: HashMap<_, _> = url.query_pairs().collect();

	let maybe_code = query.get("code");
	let maybe_state = query.get("state");

	match (maybe_code, maybe_state) {
		(Some(code), Some(state)) => {
			let maybe_value = ctx.kv(kv::LOGINS)?.get(&state).await?;
			match maybe_value {
				Some(value) => {
					let member: PartialMember = value.as_json()?;
				}
				None => {}
			}
		}
		_ => {}
	}

	todo!()
}
