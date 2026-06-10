import CloudKit
import Foundation

let CONTAINER_ID = "iCloud.com.stboyden.mpalsts"
let RECORD_ID = CKRecord.ID(recordName: "appearance-preference")

enum ThemeMode: UInt8, Codable {
	case light
	case dark
}

struct Message: Codable {
	var themeMode: ThemeMode
	var updatedTimeMs: UInt128
}

enum Errors: Int8 {
	case couldNotDecodeMessage = -4
	case couldNotEncodeMessage = -3
	case invalidThemeMode = -2
	case couldNotUpdateAppearance = -1
}

private func saveAppearanceAsync(message: Message) async -> Int8 {
	let container = CKContainer(identifier: CONTAINER_ID)
	let database = container.privateCloudDatabase

	let record = CKRecord(recordType: "AppearancePreference", recordID: RECORD_ID)

	let encoder = JSONEncoder()
	guard let encoded = try? encoder.encode(message) else {
		return Errors.couldNotEncodeMessage.rawValue
	}

	record["data"] = encoded as CKRecordValue

	do {
		_ = try await database.save(record)
		return 0
	} catch {
		return -1
	}
}

public typealias MPALSTSCloudKitSaveAppearanceCallback =
	@convention(c) (UnsafeMutableRawPointer?, Int8) -> Void

private struct SaveAppearanceCallbackBox: @unchecked Sendable {
	let callback: MPALSTSCloudKitSaveAppearanceCallback

	func call(_ context: UnsafeMutableRawPointer?, _ code: Int8) {
		callback(context, code)
	}
}

public typealias MPALSTSCloudKitGetAppearanceCallback =
	@convention(c) (UnsafeMutableRawPointer?, UInt8) -> Void

private struct GetAppearanceCallbackBox: @unchecked Sendable {
	let callback: MPALSTSCloudKitGetAppearanceCallback

	func call(_ context: UnsafeMutableRawPointer?, _ code: UInt8) {
		callback(context, code)
	}
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
public func saveAppearance(
	_ themeModeRaw: CUnsignedChar,
	_ context: sending UnsafeMutableRawPointer?,
	_ completion: @escaping MPALSTSCloudKitSaveAppearanceCallback
) {
	guard let themeMode = ThemeMode(rawValue: themeModeRaw) else {
		completion(context, Errors.invalidThemeMode.rawValue)
		return
	}

	let now = UInt128(NSDate().timeIntervalSince1970 * 1000)

	let message = Message(
		themeMode: themeMode,
		updatedTimeMs: now
	)

	let callbackBox = SaveAppearanceCallbackBox(callback: completion)

	Task.detached {
		let code = await saveAppearanceAsync(message: message)
		callbackBox.call(context, code)
	}
}

func getAppearanceAsync() async -> UInt8 {
	let container = CKContainer(identifier: CONTAINER_ID)
	let database = container.privateCloudDatabase

	guard
		let record: CKRecord =
			(try? await withCheckedThrowingContinuation { continuation in
				database.fetch(
					withRecordID: RECORD_ID,
					completionHandler: { (record: CKRecord?, error) in
						if let error {
							continuation.resume(throwing: error)
						} else {
							continuation.resume(returning: record)
						}
					})
			}),
		let data = record["data"] as? Data
	else {
		return 0
	}

	let decoder = JSONDecoder()
	guard let decoded = try? decoder.decode(Message.self, from: data) else {
		return 0
	}

	return decoded.themeMode.rawValue
}

@_cdecl("mpalsts_cloudkit_get_appearance")
public func getAppearance(
	_ context: sending UnsafeMutableRawPointer?,
	_ completion: @escaping MPALSTSCloudKitGetAppearanceCallback
) {
	let callbackBox = GetAppearanceCallbackBox(callback: completion)

	Task.detached {
		let code = await getAppearanceAsync()
		callbackBox.call(context, code)
	}
}
