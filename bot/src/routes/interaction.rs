use std::convert::TryInto;

use ed25519::Signature;
use ed25519_dalek::{PublicKey, Verifier};
use hex::decode;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use twilight_model::{
	application::{
		callback::{CallbackData, InteractionResponse},
		command::CommandOptionChoice,
		component::{button::ButtonStyle, ActionRow, Button, Component},
		interaction::{application_command::CommandDataOption, Interaction},
	},
	channel::message::{AllowedMentions, MessageFlags},
};
use worker::{Request, Response, RouteContext};

use crate::{
	constants::{command, index, kv},
	search::search,
};

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

fn get_public_key<T>(ctx: &RouteContext<T>) -> PublicKey {
	let bytes = decode(
		ctx.var("DISCORD_PUBLIC_KEY")
			.expect("DISCORD_PUBLIC_KEY env")
			.to_string()
			.as_bytes(),
	)
	.expect("public key hex");

	PublicKey::from_bytes(&bytes).unwrap()
}

fn verify_signature<T>(body: &str, req: &Request, ctx: &RouteContext<T>) -> Result<(), Response> {
	let maybe_signature = req.headers().get("x-signature-ed25519").unwrap();
	let maybe_timestamp = req.headers().get("x-signature-timestamp").unwrap();

	let verify = move || match (maybe_signature, maybe_timestamp) {
		(Some(sig_str), Some(time)) => {
			let message = time + &body;
			let sig = decode(sig_str)?.as_slice().try_into()?;
			get_public_key(ctx).verify(message.as_bytes(), &Signature::new(sig))?;
			Ok(())
		}
		_ => Err(anyhow::anyhow!("missing")),
	};

	verify().map_err(|e| Response::error(format!("Invalid signature: {}", e), 401).unwrap())
}

pub async fn interaction<T>(mut req: Request, ctx: RouteContext<T>) -> worker::Result<Response> {
	let body_str = req.text().await?;

	if let Err(e) = verify_signature(&body_str, &req, &ctx) {
		return Ok(e);
	}

	let interaction = serde_json::from_str(&body_str)?;

	match interaction {
		Interaction::Ping(_) => Response::from_json(&InteractionResponse::Pong),
		Interaction::ApplicationCommand(interaction) => match interaction.data.name.as_str() {
			command::LOGIN => {
				let kv = ctx.kv(kv::LOGINS)?;
				let id = nanoid!();

				kv.put(&id, to_string(&interaction.member)?)?
					.expiration_ttl(kv::LOGINS_TTL)
					.execute()
					.await?;

				let data = CallbackData {
					components: Some(vec![Component::ActionRow(ActionRow {
						components: vec![Component::Button(Button {
							custom_id: None,
							disabled: false,
							emoji: None,
							label: Some("Login".to_string()),
							style: ButtonStyle::Link,
							url: Some(format!(
								"{}/login?id={}",
								req.url()?.origin().ascii_serialization(),
								&id
							)),
						})],
					})]),
					content: Some("Login to bot (expires in 5 minutes)".to_string()),
					flags: Some(MessageFlags::EPHEMERAL),
					..CallbackData::default()
				};

				Response::from_json(&InteractionResponse::ChannelMessageWithSource(data))
			}
			command::SEARCH => {
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
			_ => Response::empty(),
		},
		Interaction::ApplicationCommandAutocomplete(interaction) => {
			let q = match &interaction.data.options[0] {
				CommandDataOption::Autocomplete { value, .. }
				| CommandDataOption::String { value, .. } => value,
				_ => unreachable!(),
			};

			let results = search::<InvType, _, _>(ctx.get_env(), index::ITEMS, q).await?;
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
		_ => Response::empty(),
	}
}
