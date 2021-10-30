use std::convert::TryFrom;

use serde::{de::DeserializeOwned, Deserialize};
use sled::{IVec, Tree};

fn deserialize<T>(vec: Option<IVec>) -> Result<Option<T>, bincode::Error>
where
	T: DeserializeOwned,
{
	vec.map(|value| bincode::deserialize_from(&*value))
		.transpose()
}

pub trait Stored
where
	IVec: TryFrom<Self::Key>,
{
	type Error: From<sled::Error> + From<<IVec as TryFrom<Self::Key>>::Error> + From<bincode::Error>;

	type Key;

	fn bytes(&self) -> Result<IVec, Self::Error>;

	fn from_tree(tree: &Tree, key: Self::Key) -> Result<Option<Self>, Self::Error>
	where
		Self: Sized,
		for<'de> Self: Deserialize<'de>,
	{
		let key_bytes = IVec::try_from(key)?;
		Ok(deserialize(tree.get(key_bytes)?)?)
	}

	fn into_tree(&self, tree: &Tree, key: Self::Key) -> Result<Option<Self>, Self::Error>
	where
		Self: Sized,
		for<'de> Self: Deserialize<'de>,
	{
		let key_bytes = IVec::try_from(key)?;
		let value_bytes = self.bytes()?;

		Ok(deserialize(tree.insert(key_bytes, value_bytes)?)?)
	}

	fn merge_into_tree(&self, tree: &Tree, key: Self::Key) -> Result<Option<Self>, Self::Error>
	where
		Self: Sized,
		for<'de> Self: Deserialize<'de>,
	{
		let key_bytes = IVec::try_from(key)?;
		let value_bytes = self.bytes()?;

		Ok(deserialize(tree.merge(key_bytes, value_bytes)?)?)
	}
}
