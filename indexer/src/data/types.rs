use meilisearch_sdk::document::Document;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvType {
	#[serde(rename = "typeID")]
	pub type_id: usize,
	// pub group_id: usize,
	pub type_name: String,
	pub description: Option<String>,
	// pub mass: Scientific,
	// pub volume: Scientific,
	// pub capacity: Scientific,
	// pub portion_size: usize,
	// #[serde(deserialize_with = "option_str::deserialize")]
	// pub race_id: Option<usize>,
	// pub base_price: Option<usize>,
	// pub published: bool,
	// pub market_group_id: Option<usize>,
	// pub icon_id: Option<usize>,
	// pub sound_id: Option<usize>,
	// pub graphic_id: usize,
}

impl Document for InvType {
	type UIDType = usize;

	fn get_uid(&self) -> &Self::UIDType {
		&self.type_id
	}
}
