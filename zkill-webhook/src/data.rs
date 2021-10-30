use bincode::{deserialize, serialize};

use crate::model::Subscriptions;

pub fn subscription_merge_operator(
	_key: &[u8],
	old_value: Option<&[u8]>,
	merged_bytes: &[u8],
) -> Option<Vec<u8>> {
	match old_value {
		Some(bytes) => {
			let mut current: Subscriptions = deserialize(bytes).ok()?;
			let new: Subscriptions = deserialize(merged_bytes).ok()?;

			current.extend(new);

			Some(serialize(&current).ok()?)
		}
		None => Some(merged_bytes.into()),
	}
}
