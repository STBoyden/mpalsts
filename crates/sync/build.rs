use cfg_aliases::cfg_aliases;

fn main() {
	cfg_aliases! {
		macos: { target_os = "macos" },
		cloudkit: { all(feature = "cloudkit", macos) },

		dummy: { not(any(cloudkit))}
	}
}
