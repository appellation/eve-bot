use std::{net::SocketAddr, str::from_utf8};

use anyhow::Result;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use axum::{extract::Extension, handler::post, AddExtensionLayer, Router};
use bytes::Bytes;
use futures::prelude::*;
use reqwest::Client;
use sled::{Db, IVec};
use tokio::spawn;
use tower_http::trace::TraceLayer;
use tracing::log::{error, warn};

#[derive(Debug, Clone)]
struct State {
	pub client: Client,
	pub db: Db,
}

async fn send_message(state: State, key: IVec, data: Bytes) -> Result<()> {
	let url = from_utf8(&key)?;
	let result = state.client
		.post(url)
		.body(data)
		.send()
		.await
		.and_then(|r| r.error_for_status());

	if let Err(e) = result {
		warn!("Error executing webhook: {}; removing", e);
		state.db.remove(key)?;
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
		let data: Bytes = match msg? {
			Message::Binary(bytes) => bytes.into(),
			Message::Text(data) => data.into(),
			Message::Close(_frame) => break,
			Message::Ping(_data) => {
				ws.send(Message::Pong(vec![])).await?;
				continue;
			}
			Message::Pong(_data) => continue,
		};

		for item in state.db.iter() {
			let (k, _v) = item?;
			spawn(send_message(state.clone(), k, data.clone()));
		}
	}

	Ok(())
}

async fn register_webhook(state: Extension<State>, body: String) {
	state
		.db
		.insert(body.as_str(), IVec::default())
		.unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();

	let db = sled::open("webhooks")?;
	let client = Client::new();
	let state = State { db, client };

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

	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
