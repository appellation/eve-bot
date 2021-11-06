use serde::{Deserialize, Serialize};
use twilight_model::{
	application::{
		callback::{CallbackData, InteractionResponse},
		command::CommandOptionChoice,
		interaction::{application_command::CommandDataOption, ApplicationCommand},
	},
	channel::message::AllowedMentions,
};
use worker::{Response, RouteContext};

use crate::constants::index;

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

pub async fn search(interaction: &ApplicationCommand) -> worker::Result<Response> {
	let item_id = match &interaction.data.options[0] {
		CommandDataOption::String { value, .. } => value,
		_ => unreachable!(),
	};

	let data = CallbackData {
		content: Some(format!("https://evemarketer.com/types/{}", item_id)),
		allowed_mentions: Some(AllowedMentions::default()),
		..CallbackData::default()
	};

	Response::from_json(&InteractionResponse::ChannelMessageWithSource(data))
}

pub async fn autocomplete<T>(
	ctx: RouteContext<T>,
	interaction: &ApplicationCommand,
) -> worker::Result<Response> {
	let q = match &interaction.data.options[0] {
		CommandDataOption::Autocomplete { value, .. } | CommandDataOption::String { value, .. } => {
			value
		}
		_ => unreachable!(),
	};

	let results = crate::search::search::<InvType, _, _>(ctx.get_env(), index::ITEMS, q).await?;
	let data = CallbackData {
		choices: Some(
			results
				.hits
				.iter()
				.map(|inv| CommandOptionChoice::String {
					name: inv.type_name.clone(),
					value: inv.type_id.to_string(),
				})
				.collect(),
		),
		..CallbackData::default()
	};

	Response::from_json(&InteractionResponse::ApplicationCommandAutocompleteResult(
		data,
	))
}
