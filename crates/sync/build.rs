use cfg_aliases::cfg_aliases;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	cfg_aliases! {
		macos: { target_os = "macos" },
		cloudkit: { all(feature = "cloudkit", macos) },

		dummy: { not(any(cloudkit))}
	}

	#[cfg(all(feature = "cloudkit", target_os = "macos"))]
	{
		use std::process::Command;

		let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

		let swift_package = format!("{manifest_dir}/../../swift/MPALSTSCloudKitSync");

		let status = Command::new("swift")
			.args(["build", "-c", "release"])
			.current_dir(&swift_package)
			.status()
			.expect("failed to run swift build");

		assert!(status.success(), "Swift build failed");

		println!("cargo:rustc-link-search=native={swift_package}/.build/release");
		println!("cargo:rustc-link-lib=dylib=MPALSTSCloudKitSync");
		println!("cargo:rerun-if-changed={swift_package}/Package.swift");
		println!("cargo:rerun-if-changed={swift_package}/Sources/MPALSTSCloudKitSync/FFI.swift");
	}
}
