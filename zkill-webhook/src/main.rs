use std::{env, net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use axum::{
	extract::Extension,
	handler::post,
	http::{HeaderMap, HeaderValue},
	AddExtensionLayer, Json, Router,
};
use futures::{future::try_join_all, prelude::*};
use model::{zkb::Killmail, Filters, Format, Subscription, Subscriptions};
use reqwest::{Client, StatusCode};
use serde_json::{from_slice, from_str, to_vec};
use sled::Tree;
use sled_ext::key::Key;
use tokio::spawn;
use tower_http::trace::TraceLayer;
use tracing::log::{error, warn};

mod model;

#[derive(Debug, Clone)]
struct State {
	pub client: Client,
	pub tree: Tree,
}

/// Send a message to the subscription with the killmail contents. Returns the subscription if it failed.
async fn send_message(
	state: State,
	sub: Subscription,
	km: impl AsRef<Killmail>,
) -> Result<Option<Subscription>> {
	let body = match sub.format {
		Format::Discord => format!(r#"{{"content":"{}"}}"#, km.as_ref().zkb.url).into_bytes(),
		Format::Raw => to_vec(km.as_ref())?,
	};

	let mut headers = HeaderMap::with_capacity(1);
	headers.insert("Content-Type", HeaderValue::from_static("application/json"));

	let res = state
		.client
		.post(&sub.webhook_url)
		.body(body)
		.headers(headers)
		.send()
		.await
		.and_then(|r| r.error_for_status());

	match res {
		Err(e) => {
			warn!("Error posting to webhook {}; removing", e);
			Ok(Some(sub))
		}
		Ok(_response) => Ok(None),
	}
}

async fn process_killmail(state: State, km: Killmail) -> Result<()> {
	let filters = km.filters();
	let km = Arc::new(km);

	try_join_all(filters.into_iter().map(move |filter| {
		let km = Arc::clone(&km);
		let state = state.clone();

		async move {
			let subscriptions = filter.get(&state.tree)?.unwrap_or_default();

			let msg_state = state.clone();
			let failed = try_join_all(
				subscriptions
					.into_inner()
					.into_iter()
					.map(move |sub| send_message(msg_state.clone(), sub, Arc::clone(&km))),
			)
			.await?
			.into_iter()
			.filter_map(|sub| sub)
			.collect::<Subscriptions>();

			state.tree.transaction::<_, _, Error>(move |txn| {
				let existing = filter.get(txn).unwrap().unwrap_or_default();

				// TODO: avoid cloning
				filter.insert(txn, existing.difference(&failed).cloned().collect()).unwrap();

				Ok(())
			}).unwrap();

			Ok::<_, Error>(())
		}
	}))
	.await?;

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

		spawn(process_killmail(state.clone(), km));
	}

	Ok(())
}

async fn register_webhook(state: Extension<State>, Json(body): Json<Filters>) -> StatusCode {
	let res = state
		.tree
		.transaction::<_, _, Error>(move |txn| {
			for (filter, sub) in &body {
				let mut existing = filter.get(txn).unwrap().unwrap_or_default();
				existing.extend(sub.iter().cloned());
				filter.insert(txn, existing).unwrap();
			}
			Ok(())
		});

	if let Err(e) = &res {
		error!("{}", e);
		StatusCode::INTERNAL_SERVER_ERROR
	} else {
		StatusCode::NO_CONTENT
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();

	let db = sled::open("data")?;
	let tree = db.open_tree("webhooks")?;
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
