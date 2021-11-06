use config_struct::{Error, StructOptions};

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=config.toml");
	config_struct::create_struct("config.toml", "src/config.rs", &StructOptions::default())
}
