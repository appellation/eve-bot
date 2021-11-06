use sled::{transaction::UnabortableTransactionError, IVec};

use crate::{tree::TreeLike, value::Value};

pub trait Key
where
	Self: Sized,
{
	type Value: Value;
	type Error: From<sled::Error> + From<<Self::Value as Value>::Error>;

	fn from_bytes(bytes: &IVec) -> Result<Self, Self::Error>;
	fn to_bytes(&self) -> Result<IVec, Self::Error>;

	fn get<T>(&self, tree: &T) -> Result<Option<Self::Value>, Self::Error>
	where
		T: TreeLike,
		Self::Error: From<<T as TreeLike>::Error>,
	{
		let key_bytes = self.to_bytes()?;
		Ok(tree
			.get(key_bytes)?
			.map(|val| Self::Value::from_bytes(&val))
			.transpose()?)
	}

	fn insert<T>(&self, tree: &T, value: Self::Value) -> Result<Option<Self::Value>, Self::Error>
	where
		T: TreeLike,
		Self::Error: From<<T as TreeLike>::Error>,
	{
		let key_bytes = self.to_bytes()?;
		Ok(tree
			.insert(key_bytes, value.to_bytes()?)?
			.map(|val| Self::Value::from_bytes(&val))
			.transpose()?)
	}

	fn remove<T>(&self, tree: &T) -> Result<Option<Self::Value>, Self::Error>
	where
		T: TreeLike,
		Self::Error: From<<T as TreeLike>::Error>,
	{
		let key_bytes = self.to_bytes()?;
		Ok(tree
			.remove(key_bytes)?
			.map(|val| Self::Value::from_bytes(&val))
			.transpose()?)
	}
}

// impl<T, V> Key for T
// where
// 	for<'a> IVec: From<&'a T>,
// 	for<'a> T: From<&'a IVec>
// {
// 	type Value = V;
// 	type Error = Infallible;

// 	fn from_bytes(bytes: &IVec) -> Result<Self, Self::Error> {
// 		Ok(bytes.into())
// 	}

// 	fn to_bytes(&self) -> Result<IVec, <Self as Value>::Error> {
// 		Ok(self.into())
// 	}
// }
