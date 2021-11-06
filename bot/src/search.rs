use std::{borrow::Cow, collections::HashMap, fmt::Display};

use futures::future::try_join_all;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::to_string;
use strum_macros::{Display, IntoStaticStr};
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

#[derive(
	Debug, IntoStaticStr, Display, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy,
)]
#[serde(rename_all = "snake_case")]
pub enum Category {
	// #[strum(serialize = "agent")]
	// Agent,
	#[strum(serialize = "alliance")]
	Alliance,
	#[strum(serialize = "character")]
	Character,
	// #[strum(serialize = "constellation")]
	// Constellation,
	#[strum(serialize = "corporation")]
	Corporation,
	// #[strum(serialize = "faction")]
	// Faction,
	#[strum(serialize = "inventory_type")]
	InventoryType,
	// #[strum(serialize = "region")]
	// Region,
	#[strum(serialize = "solar_system")]
	SolarSystem,
	// #[strum(serialize = "station")]
	// Station,
}

impl Category {
	const fn url_path(&self) -> &'static str {
		match self {
			Category::Alliance => "/alliances",
			Category::Character => "/characters",
			Category::Corporation => "/corporations",
			Category::InventoryType => "/universe/types",
			Category::SolarSystem => "/universe/systems",
		}
	}
}

const ESI_BASE_URL: &'static str = "https://esi.evetech.net/latest";

type EveSearchResponse = HashMap<Category, Vec<usize>>;

#[derive(Debug, Deserialize)]
struct Named {
	name: String,
}

#[derive(Debug)]
pub struct SearchData {
	pub id: usize,
	pub name: String,
}

pub async fn eve(category: Category, query: &str) -> worker::Result<Vec<SearchData>> {
	if query.chars().count() < 3 {
		return Err(worker::Error::RustError(format!(
			"Character count less than 3! ({})",
			query
		)));
	}

	let request = Fetch::Request(Request::new(
		&format!(
			"{}/search/?search={}&categories={}",
			ESI_BASE_URL, query, category
		),
		Method::Get,
	)?);

	let mut response = request.send().await?;
	let body: EveSearchResponse = match response.status_code() {
		200..=299 => Ok(response.json().await?),
		_ => Err(worker::Error::RustError(format!(
			"Bad search response! {}",
			response.text().await?
		))),
	}?;

	let futures = body[&category].iter().take(45).map(|id| async move {
		let request = Fetch::Request(Request::new(
			&format!("{}{}/{}", ESI_BASE_URL, category.url_path(), id),
			Method::Get,
		)?);

		let response: Named = request.send().await?.json().await?;

		Ok::<_, worker::Error>(SearchData {
			id: *id,
			name: response.name,
		})
	});

	Ok(try_join_all(futures).await?)
}
