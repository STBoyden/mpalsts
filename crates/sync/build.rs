use cfg_aliases::cfg_aliases;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	cfg_aliases! {
		macos: { target_os = "macos" },
		cloudkit: { all(feature = "cloudkit", macos) },

		dummy: { not(any(cloudkit))}
	}
}
