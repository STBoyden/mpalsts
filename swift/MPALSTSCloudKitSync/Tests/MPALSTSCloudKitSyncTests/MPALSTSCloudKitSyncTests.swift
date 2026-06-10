import Foundation
import Testing

@testable import MPALSTSCloudKitSync

private let callbackLock = NSLock()
nonisolated(unsafe) private var invalidThemeCallbackResult: (UnsafeMutableRawPointer?, Int8)?
nonisolated(unsafe) private var saveAppearanceCallbackResult: Int8?
nonisolated(unsafe) private var getAppearanceCallbackResult: UInt8?
nonisolated(unsafe) private var saveAppearanceSemaphore: DispatchSemaphore?
nonisolated(unsafe) private var getAppearanceSemaphore: DispatchSemaphore?

private func invalidThemeCallback(_ context: UnsafeMutableRawPointer?, _ code: Int8) {
	callbackLock.lock()
	invalidThemeCallbackResult = (context, code)
	callbackLock.unlock()
}

private func resetInvalidThemeCallbackResult() {
	callbackLock.lock()
	invalidThemeCallbackResult = nil
	callbackLock.unlock()
}

private func readInvalidThemeCallbackResult() -> (UnsafeMutableRawPointer?, Int8)? {
	callbackLock.lock()
	let result = invalidThemeCallbackResult
	callbackLock.unlock()
	return result
}

private func liveSaveAppearanceCallback(_ context: UnsafeMutableRawPointer?, _ code: Int8) {
	#expect(context == nil)

	callbackLock.lock()
	saveAppearanceCallbackResult = code
	let semaphore = saveAppearanceSemaphore
	callbackLock.unlock()

	semaphore?.signal()
}

private func liveGetAppearanceCallback(_ context: UnsafeMutableRawPointer?, _ code: UInt8) {
	#expect(context == nil)

	callbackLock.lock()
	getAppearanceCallbackResult = code
	let semaphore = getAppearanceSemaphore
	callbackLock.unlock()

	semaphore?.signal()
}

private func resetLiveCallbackResults(saveSemaphore: DispatchSemaphore, getSemaphore: DispatchSemaphore) {
	callbackLock.lock()
	saveAppearanceCallbackResult = nil
	getAppearanceCallbackResult = nil
	saveAppearanceSemaphore = saveSemaphore
	getAppearanceSemaphore = getSemaphore
	callbackLock.unlock()
}

private func readLiveSaveAppearanceCallbackResult() -> Int8? {
	callbackLock.lock()
	let result = saveAppearanceCallbackResult
	callbackLock.unlock()
	return result
}

private func readLiveGetAppearanceCallbackResult() -> UInt8? {
	callbackLock.lock()
	let result = getAppearanceCallbackResult
	callbackLock.unlock()
	return result
}

@Test func themeModeRawValuesMatchRustFFIContract() {
	#expect(ThemeMode.light.rawValue == 0)
	#expect(ThemeMode.dark.rawValue == 1)
	#expect(ThemeMode(rawValue: 2) == nil)
}

@Test func debugBuildUsesDevelopmentCloudKitContainer() {
	#if DEBUG
	#expect(CONTAINER_ID == "iCloud.com.stboyden.mpalsts.dev")
	#else
	#expect(CONTAINER_ID == "iCloud.com.stboyden.mpalsts")
	#endif
}

@Test func messageRoundTripsThroughJSON() throws {
	let message = Message(themeMode: .dark, updatedTimeMs: 1_717_171_717_171)

	let encoded = try JSONEncoder().encode(message)
	let decoded = try JSONDecoder().decode(Message.self, from: encoded)

	#expect(decoded.themeMode == .dark)
	#expect(decoded.updatedTimeMs == 1_717_171_717_171)
}

@Test func saveAppearanceReturnsInvalidThemeModeWithoutCloudKitCall() {
	resetInvalidThemeCallbackResult()

	saveAppearance(2, nil, invalidThemeCallback)

	let result = readInvalidThemeCallbackResult()
	#expect(result?.0 == nil)
	#expect(result?.1 == Errors.invalidThemeMode.rawValue)
}

@Test func cloudKitErrorCodesMatchRustFFIContract() {
	#expect(Errors.couldNotUpdateAppearance.rawValue == -1)
	#expect(Errors.invalidThemeMode.rawValue == -2)
	#expect(Errors.couldNotEncodeMessage.rawValue == -3)
	#expect(Errors.couldNotDecodeMessage.rawValue == -4)
}

@Test func liveCloudKitPushesDarkAppearanceAndReadsItBack() throws {
	guard ProcessInfo.processInfo.environment["MPALSTS_CLOUDKIT_LIVE_TESTS"] == "1" else {
		return
	}

	let saveSemaphore = DispatchSemaphore(value: 0)
	let getSemaphore = DispatchSemaphore(value: 0)
	resetLiveCallbackResults(saveSemaphore: saveSemaphore, getSemaphore: getSemaphore)

	saveAppearance(ThemeMode.dark.rawValue, nil, liveSaveAppearanceCallback)
	#expect(saveSemaphore.wait(timeout: .now() + 30) == .success)
	#expect(readLiveSaveAppearanceCallbackResult() == 0)

	getAppearance(nil, liveGetAppearanceCallback)
	#expect(getSemaphore.wait(timeout: .now() + 30) == .success)
	#expect(readLiveGetAppearanceCallbackResult() == ThemeMode.dark.rawValue)
}
