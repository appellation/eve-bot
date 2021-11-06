use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use meilisearch_sdk::{client::Client, progress::UpdateStatus};
use structopt::StructOpt;
use tokio::time::sleep;

use crate::{config::CONFIG, data::TYPES_SEARCH};

mod config;
mod data;

#[derive(Debug, StructOpt)]
struct Opt {
	/// The search index to use.
	index: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let opt = Opt::from_args();
	let search = Client::new(CONFIG.search.api_url, CONFIG.search.api_keys.private);

	let bar = ProgressBar::new(5)
		.with_message("Processing EVE items...")
		.with_style(
			ProgressStyle::default_bar()
				.template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {msg}")
				.progress_chars("#>-"),
		);
	bar.enable_steady_tick(120);

	bar.set_message("Uploading...");
	bar.inc(1);

	let progress = match opt.index.as_str() {
		"types" => TYPES_SEARCH.data()?.insert(&search).await?,
		_ => panic!(),
	};

	bar.inc(1);

	loop {
		let status = progress.get_status().await?;

		match status {
			UpdateStatus::Enqueued { .. } => {
				bar.set_position(3);
				bar.set_message("Index queued...");
			}
			UpdateStatus::Failed { content } => {
				bar.abandon_with_message(format!("Indexing failed: {:?}", content.error));
				break;
			}
			UpdateStatus::Processed { .. } => {
				bar.set_message("Indexing complete");
				bar.set_position(5);
				break;
			}
			UpdateStatus::Processing { .. } => {
				bar.set_message("Processing...");
				bar.set_position(4);
			}
		}

		sleep(Duration::from_millis(200)).await;
	}

	Ok(())
}
