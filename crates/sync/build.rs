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
		let cargo_profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
		let swift_configuration = if cargo_profile == "release" {
			"release"
		} else {
			"debug"
		};

		let status = Command::new("swift")
			.args(["build", "-c", swift_configuration])
			.current_dir(&swift_package)
			.status()
			.expect("failed to run swift build");

		assert!(status.success(), "Swift build failed");

		let swift_library_dir = format!("{swift_package}/.build/{swift_configuration}");

		println!("cargo:rustc-link-search=native={swift_library_dir}");
		println!("cargo:rustc-link-lib=static=MPALSTSCloudKitSync");
		println!("cargo:rustc-link-lib=framework=CloudKit");
		println!("cargo:rustc-link-lib=framework=Foundation");
		println!("cargo:rustc-link-arg=-mmacosx-version-min=15.0");
		println!("cargo:rerun-if-changed={swift_package}/Package.swift");
		println!("cargo:rerun-if-changed={swift_package}/Sources/MPALSTSCloudKitSync/FFI.swift");
	}
}
