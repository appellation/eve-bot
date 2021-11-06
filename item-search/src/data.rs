use std::{marker::PhantomData, path::Path};

use anyhow::Result;
use csv::Reader;
use meilisearch_sdk::{client::Client, document::Document, progress::Progress};
use serde::Deserialize;

use self::types::InvType;

pub mod types;

trait Data
where
	Self: for<'de> Deserialize<'de>,
{
	fn read<P: AsRef<Path>>(path: P) -> Result<Vec<Self>> {
		let mut reader = Reader::from_path(path)?;

		Ok(reader
			.deserialize::<Self>()
			.collect::<Result<Vec<_>, _>>()?)
	}
}

impl<T> Data for T where T: for<'de> Deserialize<'de> {}

pub struct SearchData<'a, T> {
	search: &'a Search<T>,
	data: Vec<T>,
}

impl<'a, T> SearchData<'a, T>
where
	T: Document,
{
	pub async fn insert(&self, client: &Client) -> Result<Progress> {
		Ok(client
			.get_or_create(self.search.index)
			.await?
			.add_documents(&self.data, self.search.primary_key)
			.await?)
	}
}

#[derive(Debug, Clone)]
pub struct Search<T> {
	pub index: &'static str,
	pub primary_key: Option<&'static str>,
	pub path: &'static str,
	contents: PhantomData<T>,
}

impl<T> Search<T>
where
	T: for<'de> Deserialize<'de>,
{
	pub fn data(&self) -> Result<SearchData<T>> {
		let data = T::read(self.path)?;
		Ok(SearchData { search: self, data })
	}
}

pub const TYPES_SEARCH: Search<InvType> = Search {
	index: "types",
	primary_key: Some("typeID"),
	path: "./data/invTypes.csv",
	contents: PhantomData,
};
