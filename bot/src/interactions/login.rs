use nanoid::nanoid;
use serde_json::to_string;
use twilight_model::{
	application::{
		callback::{CallbackData, InteractionResponse},
		component::{button::ButtonStyle, ActionRow, Button, Component},
		interaction::ApplicationCommand,
	},
	channel::message::MessageFlags,
};
use worker::{Request, Response, RouteContext};

use crate::constants::kv;

pub async fn login<T>(
	req: Request,
	ctx: RouteContext<T>,
	interaction: &ApplicationCommand,
) -> worker::Result<Response> {
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
