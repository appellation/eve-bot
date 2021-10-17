use std::{borrow::Cow, fmt::Display};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::to_string;
use worker::{
	wasm_bindgen::JsValue, CfProperties, Env, Fetch, Method, Request, RequestInit, RequestRedirect,
};

use crate::constants::var;

#[derive(Debug, Serialize, Deserialize)]
struct SearchRequest<'a> {
	q: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse<T> {
	pub hits: Vec<T>,
}

pub async fn search<'a, T, I, Q>(env: Env, index: I, query: Q) -> worker::Result<SearchResponse<T>>
where
	T: DeserializeOwned,
	I: Display,
	Q: Into<Cow<'a, str>> + Clone,
{
	let base = env.var(var::MEILISEARCH_API_URL)?.to_string();

	let headers = [
		(
			"X-Meili-API-Key",
			env.secret(var::MEILISEARCH_API_KEY)?.to_string().as_str(),
		),
		("Content-Type", "application/json"),
	]
	.iter()
	.collect();

	let request = Fetch::Request(Request::new_with_init(
		&format!("{}/indexes/{}/search", base, index),
		&RequestInit {
			body: Some(JsValue::from_str(&to_string(&SearchRequest {
				q: query.into(),
			})?)),
			headers,
			cf: CfProperties::default(),
			method: Method::Post,
			redirect: RequestRedirect::default(),
		},
	)?);

	let mut response = request.send().await?;
	match response.status_code() {
		200..=299 => Ok(response.json().await?),
		_ => Err(worker::Error::RustError(format!(
			"Bad search response! {}",
			response.text().await?
		))),
	}
}
