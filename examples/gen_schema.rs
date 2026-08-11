use ptr::config::Config;
use schemars::schema_for;

fn main() {
	let schema = schema_for!(Config);
	std::fs::write(
		"examples/schema.json",
		serde_json::to_string_pretty(&schema).unwrap(),
	)
	.unwrap();
}
