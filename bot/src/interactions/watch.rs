use twilight_model::application::interaction::{
	application_command::CommandDataOption, ApplicationCommand,
};
use worker::{Response, RouteContext};

pub async fn watch<T>(
	ctx: RouteContext<T>,
	interaction: &ApplicationCommand,
) -> worker::Result<Response> {
	let webhook = match &interaction.data.options[0] {
		CommandDataOption::Autocomplete { value, .. } | CommandDataOption::String { value, .. } => {
			value
		}
		_ => unreachable!(),
	};

	// ctx.kv(binding);

	todo!()
}
