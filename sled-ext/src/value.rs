use std::convert::Infallible;

use sled::IVec;

pub trait Value
where
	Self: Sized,
{
	type Error;

	fn from_bytes(bytes: &IVec) -> Result<Self, Self::Error>;
	fn to_bytes(&self) -> Result<IVec, Self::Error>;
}

impl<T> Value for T
where
	for<'a> IVec: From<&'a T>,
	for<'a> T: From<&'a IVec>,
{
	type Error = Infallible;

	fn from_bytes(bytes: &IVec) -> Result<Self, Self::Error> {
		Ok(bytes.into())
	}

	fn to_bytes(&self) -> Result<IVec, <Self as Value>::Error> {
		Ok(self.into())
	}
}

pub trait ValueContainer {
	//
}
