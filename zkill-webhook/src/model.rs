use std::{
	collections::{HashMap, HashSet},
	convert::TryFrom,
};

use anyhow::{Context, Error, Result};
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use sled::IVec;

use crate::prelude::*;

pub mod zkb;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
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

impl TryFrom<Filter> for IVec {
	type Error = Error;

	fn try_from(value: Filter) -> Result<Self, Self::Error> {
		Ok(serialize(&value)
			.context("Failed to serialize Filter into CBOR")?
			.into())
	}
}

impl TryFrom<IVec> for Filter {
	type Error = Error;

	fn try_from(value: IVec) -> Result<Self, Self::Error> {
		deserialize(&*value).context("Failed to read Filter from CBOR")
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Subscription {
	pub webhook_url: String,
	pub format: Format,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Format {
	Raw,
	Discord,
}

pub type Subscriptions = HashSet<Subscription>;

impl Stored for Subscriptions {
	type Error = Error;
	type Key = Filter;

	fn bytes(&self) -> Result<IVec> {
		Ok(serialize(&self)
			.context("Failed to serialize self into CBOR")?
			.into())
	}
}

pub type Filters = HashMap<Filter, Subscriptions>;
