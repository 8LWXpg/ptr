use std::collections::HashSet;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pin(Option<HashSet<String>>);

impl Pin {
	pub fn add(&mut self, names: Vec<String>) {
		if let Some(pins) = self.0.as_mut() {
			names.into_iter().for_each(|n| {
				pins.insert(n);
			});
		} else {
			self.0 = Some(HashSet::from_iter(names));
		}
	}

	pub fn remove(&mut self, names: Vec<String>) {
		if let Some(pins) = self.0.as_mut() {
			names.iter().for_each(|n| {
				pins.remove(n);
			});
		} else {
			self.0 = Some(HashSet::from_iter(names));
		}
	}

	pub fn list(&self) {
		if let Some(pins) = self.0.as_ref() {
			pins.iter().for_each(|n| println!("{n}"));
		}
	}

	pub fn reset(&mut self) {
		self.0 = None;
	}

	pub fn contains(&self, value: &str) -> bool {
		match &self.0 {
			Some(pins) => pins.contains(value),
			None => false,
		}
	}
}

impl Default for Pin {
	fn default() -> Self {
		Self(None)
	}
}
