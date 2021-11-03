use sled::{
	transaction::{TransactionalTree, UnabortableTransactionError},
	IVec, Tree,
};

pub trait TreeLike {
	type Error;

	fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<IVec>, Self::Error>;
	fn insert<K, V>(&self, key: K, value: V) -> Result<Option<IVec>, Self::Error>
	where
		K: AsRef<[u8]> + Into<IVec>,
		V: Into<IVec>;
	fn remove<K: AsRef<[u8]> + Into<IVec>>(&self, key: K) -> Result<Option<IVec>, Self::Error>;
}

impl TreeLike for Tree {
	type Error = sled::Error;

	fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<IVec>, Self::Error> {
		self.get(key)
	}

	fn insert<K, V>(&self, key: K, value: V) -> Result<Option<IVec>, Self::Error>
	where
		K: AsRef<[u8]>,
		V: Into<IVec>,
	{
		self.insert(key, value)
	}

	fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<IVec>, Self::Error> {
		self.remove(key)
	}
}

impl TreeLike for TransactionalTree {
	type Error = UnabortableTransactionError;

	fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<IVec>, Self::Error> {
		self.get(key)
	}

	fn insert<K, V>(&self, key: K, value: V) -> Result<Option<IVec>, Self::Error>
	where
		K: AsRef<[u8]> + Into<IVec>,
		V: Into<IVec>,
	{
		self.insert(key, value)
	}

	fn remove<K: AsRef<[u8]> + Into<IVec>>(&self, key: K) -> Result<Option<IVec>, Self::Error> {
		self.remove(key)
	}
}
