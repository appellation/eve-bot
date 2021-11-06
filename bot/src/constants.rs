pub mod command {
	pub const LOGIN: &'static str = "login";
	pub const ITEM: &'static str = "item";
	pub const WATCH: &'static str = "watch";
}

pub mod kv {
	pub const LOGINS: &'static str = "LOGINS";
	pub const LOGINS_TTL: u64 = 60 * 5;
}

pub mod var {
	pub const MEILISEARCH_API_KEY: &'static str = "MEILISEARCH_API_KEY";
	pub const MEILISEARCH_API_URL: &'static str = "MEILISEARCH_API_URL";
}

pub mod index {
	pub const ITEMS: &'static str = "types";
}
