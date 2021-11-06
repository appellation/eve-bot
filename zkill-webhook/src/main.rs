use std::{env, net::SocketAddr};

use anyhow::{Error, Result};
use axum::{extract::Extension, routing::post, AddExtensionLayer, Router};
use axum_msgpack::MsgPack;
use model::Filters;
use reqwest::{Client, StatusCode};
use sled::Tree;
use sled_ext::key::Key;
use tokio::spawn;
use tower_http::trace::TraceLayer;
use tracing::log::error;

mod model;
mod ws;

#[derive(Debug, Clone)]
pub struct State {
	pub client: Client,
	pub tree: Tree,
}

async fn register_webhook(state: Extension<State>, MsgPack(body): MsgPack<Filters>) -> StatusCode {
	if body.is_empty() {
		return StatusCode::BAD_REQUEST;
	}

	let res = state.tree.transaction::<_, _, Error>(move |txn| {
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
			if let Err(e) = ws::run(ws_state.clone()).await {
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
