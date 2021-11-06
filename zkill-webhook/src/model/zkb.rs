use serde::{Deserialize, Serialize};

use super::{Filter, Involvement, Role};

// {"attackers":[{"alliance_id":99008829,"character_id":95990061,"corporation_id":98675241,"damage_done":2604,"final_blow":true,"security_status":-1.1,"ship_type_id":17709,"weapon_type_id":3512}],"killmail_id":96215665,"killmail_time":"2021-10-28T04:51:31Z","solar_system_id":30004979,"victim":{"alliance_id":99007969,"character_id":2119260464,"corporation_id":98536418,"damage_taken":2604,"items":[{"flag":11,"item_type_id":22291,"quantity_dropped":1,"singleton":0},{"flag":93,"item_type_id":31788,"quantity_destroyed":1,"singleton":0},{"flag":20,"item_type_id":380,"quantity_destroyed":1,"singleton":0},{"flag":27,"item_type_id":10631,"quantity_destroyed":1,"singleton":0},{"flag":30,"item_type_id":24473,"quantity_destroyed":33,"singleton":0},{"flag":19,"item_type_id":5973,"quantity_dropped":1,"singleton":0},{"flag":29,"item_type_id":24473,"quantity_destroyed":33,"singleton":0},{"flag":5,"item_type_id":24479,"quantity_dropped":2000,"singleton":0},{"flag":22,"item_type_id":448,"quantity_dropped":1,"singleton":0},{"flag":28,"item_type_id":10631,"quantity_destroyed":1,"singleton":0},{"flag":5,"item_type_id":24475,"quantity_destroyed":2160,"singleton":0},{"flag":30,"item_type_id":10631,"quantity_dropped":1,"singleton":0},{"flag":29,"item_type_id":10631,"quantity_destroyed":1,"singleton":0},{"flag":27,"item_type_id":24473,"quantity_dropped":33,"singleton":0},{"flag":28,"item_type_id":24473,"quantity_destroyed":33,"singleton":0},{"flag":12,"item_type_id":22291,"quantity_destroyed":1,"singleton":0},{"flag":92,"item_type_id":31788,"quantity_destroyed":1,"singleton":0},{"flag":5,"item_type_id":24473,"quantity_dropped":1800,"singleton":0},{"flag":94,"item_type_id":26929,"quantity_destroyed":1,"singleton":0},{"flag":21,"item_type_id":4027,"quantity_destroyed":1,"singleton":0}],"position":{"x":1398830485426.5562,"y":283874500452.13007,"z":919633873008.7272},"ship_type_id":602},"zkb":{"locationID":40315274,"hash":"cff36d79e4b17b6eca051a08b38a1b22170670dd","fittedValue":7777534.15,"droppedValue":4049768.47,"destroyedValue":4088340.88,"totalValue":8138109.35,"points":5,"npc":false,"solo":true,"awox":false,"esi":"https:\/\/esi.evetech.net\/latest\/killmails\/96215665\/cff36d79e4b17b6eca051a08b38a1b22170670dd\/","url":"https:\/\/zkillboard.com\/kill\/96215665\/"}}

#[derive(Debug, Serialize, Deserialize)]
pub struct Killmail {
	pub attackers: Vec<Attacker>,
	pub killmail_id: usize,
	pub killmail_time: String,
	pub solar_system_id: usize,
	pub victim: Victim,
	pub zkb: Zkb,
}

impl Killmail {
	// TODO: optimize with iters
	pub fn filters(&self) -> Vec<Filter> {
		let mut filters = vec![Filter::All, Filter::System(self.solar_system_id)];
		filters.extend(
			self.attackers
				.iter()
				.flat_map(|attacker| attacker.filters()),
		);
		filters.extend(self.victim.filters());
		filters
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Victim {
	pub alliance_id: Option<usize>,
	pub character_id: Option<usize>,
	pub corporation_id: usize,
	pub damage_taken: usize,
	// pub items
	// pub position
	pub ship_type_id: usize,
}

impl Victim {
	pub fn filters(&self) -> Vec<Filter> {
		let mut filters = Vec::with_capacity(5);

		filters.push(Filter::Ship(Involvement {
			id: self.ship_type_id,
			role: Role::Attacker,
		}));

		if let Some(char_id) = self.character_id {
			filters.push(Filter::Character(Involvement {
				id: char_id,
				role: Role::Attacker,
			}));
		}

		if let Some(alliance_id) = self.alliance_id {
			filters.push(Filter::Alliance(Involvement {
				id: alliance_id,
				role: Role::Attacker,
			}));
		}

		filters.push(Filter::Corporation(Involvement {
			id: self.corporation_id,
			role: Role::Attacker,
		}));

		filters
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attacker {
	pub alliance_id: Option<usize>,
	pub character_id: Option<usize>,
	pub corporation_id: Option<usize>,
	pub damage_done: usize,
	pub final_blow: bool,
	pub security_status: f32,
	pub ship_type_id: Option<usize>,
	pub weapon_type_id: Option<usize>,
}

impl Attacker {
	pub fn filters(&self) -> Vec<Filter> {
		let mut filters = Vec::with_capacity(5);

		if let Some(ship_id) = self.ship_type_id {
			filters.push(Filter::Ship(Involvement {
				id: ship_id,
				role: Role::Attacker,
			}));
		}

		if let Some(char_id) = self.character_id {
			filters.push(Filter::Character(Involvement {
				id: char_id,
				role: Role::Attacker,
			}))
		}

		if let Some(alliance_id) = self.alliance_id {
			filters.push(Filter::Alliance(Involvement {
				id: alliance_id,
				role: Role::Attacker,
			}));
		}

		if let Some(corp_id) = self.corporation_id {
			filters.push(Filter::Corporation(Involvement {
				id: corp_id,
				role: Role::Attacker,
			}));
		}

		filters
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zkb {
	pub url: String,
}
