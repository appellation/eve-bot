use worker::{
	console_log, event, wasm_bindgen, wasm_bindgen_futures, worker_sys, Date, Env, Request,
	Response, Router,
};

mod constants;
mod interactions;
mod routes;
mod search;
mod utils;

fn log_request(req: &Request) {
	console_log!(
		"{} - [{}], located at: {:?}, within: {}",
		Date::now().to_string(),
		req.path(),
		req.cf().coordinates().unwrap_or_default(),
		req.cf().region().unwrap_or("unknown region".into())
	);
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> worker::Result<Response> {
	log_request(&req);

	utils::set_panic_hook();

	let router = Router::new();

	router
		.post_async("/api/interact", routes::interaction::interaction)
		.get_async("/login", routes::login::login)
		.get_async("/login/callback", routes::login::callback)
		.run(req, env)
		.await
}
