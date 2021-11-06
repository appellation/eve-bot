use twilight_model::{
	application::{
		callback::{CallbackData, InteractionResponse},
		command::CommandOptionChoice,
		interaction::{application_command::CommandDataOption, ApplicationCommand},
	},
	channel::message::AllowedMentions,
};
use worker::Response;

use crate::search::Category;

pub async fn search(interaction: &ApplicationCommand) -> worker::Result<Response> {
	let alliance_id = match &interaction.data.options[0] {
		CommandDataOption::String { value, .. } => value,
		_ => unreachable!(),
	};

	let data = CallbackData {
		content: Some(format!("https://zkillboard.com/alliance/{}", alliance_id)),
		allowed_mentions: Some(AllowedMentions::default()),
		..CallbackData::default()
	};

	Response::from_json(&InteractionResponse::ChannelMessageWithSource(data))
}

pub async fn autocomplete(interaction: &ApplicationCommand) -> worker::Result<Response> {
	let q = match &interaction.data.options[0] {
		CommandDataOption::Autocomplete { value, .. } | CommandDataOption::String { value, .. } => {
			value
		}
		_ => unreachable!(),
	};

	let results = crate::search::eve(Category::Alliance, q).await?;
	let data = CallbackData {
		choices: Some(
			results
				.into_iter()
				.map(|inv| CommandOptionChoice::String {
					name: inv.name,
					value: inv.id.to_string(),
				})
				.collect(),
		),
		..CallbackData::default()
	};

	Response::from_json(&InteractionResponse::ApplicationCommandAutocompleteResult(
		data,
	))
}
