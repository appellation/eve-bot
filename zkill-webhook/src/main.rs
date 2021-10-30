use std::{env, net::SocketAddr, sync::Arc};

use anyhow::Result;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use axum::{
	extract::Extension,
	handler::post,
	http::{HeaderMap, HeaderValue},
	AddExtensionLayer, Json, Router,
};
use futures::prelude::*;
use model::{zkb::Killmail, Filters, Format, Subscription, Subscriptions};
use prelude::Stored;
use reqwest::{Client, StatusCode};
use serde_json::{from_slice, from_str, to_vec};
use sled::Tree;
use tokio::spawn;
use tower_http::trace::TraceLayer;
use tracing::log::{error, warn};

mod data;
mod model;
mod prelude;

#[derive(Debug, Clone)]
struct State {
	pub client: Client,
	pub tree: Tree,
}

async fn send_message(state: State, sub: Subscription, km: impl AsRef<Killmail>) -> Result<()> {
	let body = match sub.format {
		Format::Discord => format!(r#"{{"content":"{}"}}"#, km.as_ref().zkb.url).into_bytes(),
		Format::Raw => to_vec(km.as_ref())?,
	};

	let mut headers = HeaderMap::with_capacity(1);
	headers.insert("Content-Type", HeaderValue::from_static("application/json"));

	let result = state
		.client
		.post(&sub.webhook_url)
		.body(body)
		.headers(headers)
		.send()
		.await
		.and_then(|r| r.error_for_status());

	if let Err(e) = result {
		warn!("Error executing webhook: {}; removing", e);
		// state.db.remove(key)?;
	}

	Ok(())
}

async fn run_ws(state: State) -> Result<()> {
	let (mut ws, _res) = connect_async("wss://zkillboard.com/websocket/").await?;
	ws.send(Message::Text(
		r#"{"action":"sub","channel":"killstream"}"#.into(),
	))
	.await?;

	while let Some(msg) = ws.next().await {
		let km: Killmail = match msg? {
			Message::Binary(bytes) => from_slice(&bytes)?,
			Message::Text(data) => from_str(&data)?,
			Message::Close(_frame) => break,
			Message::Ping(_data) => {
				ws.send(Message::Pong(vec![])).await?;
				continue;
			}
			Message::Pong(_data) => continue,
		};

		let filters = km.filters();
		let km = Arc::new(km);

		for filter in filters {
			let subscriptions = Subscriptions::from_tree(&state.tree, filter)?.unwrap_or_default();
			for subscription in subscriptions {
				spawn(send_message(state.clone(), subscription, Arc::clone(&km)));
			}
		}
	}

	Ok(())
}

async fn register_webhook(state: Extension<State>, Json(body): Json<Filters>) -> StatusCode {
	for (filter, sub) in body {
		let res = sub.merge_into_tree(&state.tree, filter);
		if let Err(e) = res {
			error!("{:?}", e);
			return StatusCode::INTERNAL_SERVER_ERROR;
		}
	}

	StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();

	let db = sled::open("data")?;
	let tree = db.open_tree("webhooks")?;
	tree.set_merge_operator(data::subscription_merge_operator);
	let client = Client::new();
	let state = State { tree, client };

	let ws_state = state.clone();
	spawn(async move {
		loop {
			if let Err(e) = run_ws(ws_state.clone()).await {
				error!("{}", e);
			}
		}
	});

	let app = Router::new()
		.route("/", post(register_webhook))
		.layer(TraceLayer::new_for_http())
		.layer(AddExtensionLayer::new(state));

	let addr = SocketAddr::from((
		[0, 0, 0, 0],
		env::var("PORT")
			.ok()
			.and_then(|port| port.parse().ok())
			.unwrap_or(3000),
	));

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
