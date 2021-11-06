use std::convert::TryInto;

use ed25519::Signature;
use ed25519_dalek::{PublicKey, Verifier};
use hex::decode;
use twilight_model::application::{callback::InteractionResponse, interaction::Interaction};
use worker::{Request, Response, RouteContext};

use crate::{constants::command, interactions};

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
			command::LOGIN => interactions::login(req, ctx, &interaction).await,
			command::SEARCH => interactions::search(&interaction).await,
			command::WATCH => interactions::watch(ctx, &interaction).await,
			_ => Response::empty(),
		},
		Interaction::ApplicationCommandAutocomplete(interaction) => {
			match interaction.data.name.as_str() {
				command::SEARCH => interactions::autocomplete(ctx, &interaction).await,
				_ => Response::empty(),
			}
		}
		_ => Response::empty(),
	}
}
