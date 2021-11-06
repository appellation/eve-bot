use std::{
	collections::{HashMap, HashSet},
	convert::TryFrom,
	hash::Hash,
	iter::FromIterator,
	ops::{Deref, DerefMut},
};

use anyhow::{Context, Error, Result};
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use sled::IVec;
use sled_ext::{key::Key, value::Value};

pub mod zkb;

// { { "hello": "world" }: { "foo": "bar" } }

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(tag = "type")]
pub enum Filter {
	All,
	Character(Involvement),
	Corporation(Involvement),
	Alliance(Involvement),
	System(usize),
	Ship(Involvement),
}

impl Default for Filter {
	fn default() -> Self {
		Self::All
	}
}

impl Key for Filter {
	type Value = Subscriptions;

	type Error = Error;

	fn from_bytes(bytes: &IVec) -> Result<Self, Self::Error> {
		Ok(bincode::deserialize(bytes)?)
	}

	fn to_bytes(&self) -> Result<IVec, Self::Error> {
		Ok(bincode::serialize(self)?.into())
	}
}

impl TryFrom<Filter> for IVec {
	type Error = Error;

	fn try_from(value: Filter) -> Result<Self, Self::Error> {
		Ok(serialize(&value)
			.context("Failed to serialize Filter into bincode")?
			.into())
	}
}

impl TryFrom<IVec> for Filter {
	type Error = Error;

	fn try_from(value: IVec) -> Result<Self, Self::Error> {
		deserialize(&*value).context("Failed to read Filter from bincode")
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Involvement {
	pub id: usize,
	pub role: Role,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Role {
	Attacker,
	Victim,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Subscription {
	pub webhook_url: String,
	pub format: Format,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Format {
	Raw,
	Discord,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Subscriptions(HashSet<Subscription>);

impl Subscriptions {
	pub fn into_inner(self) -> HashSet<Subscription> {
		self.0
	}
}

impl Deref for Subscriptions {
	type Target = HashSet<Subscription>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Subscriptions {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Value for Subscriptions {
	type Error = Error;

	fn from_bytes(bytes: &IVec) -> Result<Self, Self::Error> {
		Ok(bincode::deserialize(bytes)?)
	}

	fn to_bytes(&self) -> Result<IVec, Self::Error> {
		Ok(bincode::serialize(self)?.into())
	}
}

impl FromIterator<Subscription> for Subscriptions {
	fn from_iter<T: IntoIterator<Item = Subscription>>(iter: T) -> Self {
		Subscriptions(HashSet::from_iter(iter))
	}
}

// impl Stored for Subscriptions {
// 	type Error = Error;
// 	type Key = Filter;

// 	fn bytes(&self) -> Result<IVec> {
// 		Ok(serialize(&self)
// 			.context("Failed to serialize self into bincode")?
// 			.into())
// 	}

// 	// fn diff<'a>(&self, other: &'a Self) -> Self {
// 	// 	self.difference(other).collect()
// 	// }
// }

pub type Filters = HashMap<Filter, Subscriptions>;
