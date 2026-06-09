import CloudKit
import Foundation

enum ThemeMode: UInt8, Codable {
	case light
	case dark
}

struct Message: Codable {
	var themeMode: ThemeMode
	var updatedTimeMs: UInt128
}

enum Errors: Int8 {
	case invalidThemeMode = -2
	case couldNotUpdateAppearance = -1
}

/// Saves the user's appearance preference to the cloud.
///
/// ## Error codes
///
/// - '-1': Could not update the appearance preference to the cloud.
/// - '-2': The theme mode is invalid.
///
/// - Parameter themeModeRaw: The theme mode to save, as a raw `CUnsignedChar` value.
/// - Returns: `0` on success, or one of the specified negative error codes on failure.
@_cdecl("mpalsts_cloudkit_save_appearance")
public func saveAppearance(_ themeModeRaw: CUnsignedChar) -> CSignedChar {
	guard let themeMode = ThemeMode(rawValue: themeModeRaw) else {
		return Errors.invalidThemeMode.rawValue
	}

	let now = UInt128(NSDate().timeIntervalSince1970 * 1000)

	let message = Message(
		themeMode: themeMode,
		updatedTimeMs: now
	)

	return 0
}
