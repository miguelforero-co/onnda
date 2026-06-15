import Foundation
import Speech
import AVFoundation

// Voz Local ASR sidecar — transcribe an audio file with Apple SpeechAnalyzer
// (macOS 26, on-device, Neural Engine). Invoked by the Rust backend via the
// tauri-plugin-shell sidecar API.
//
// Usage:  asr <path-to-audio> [locale]
//   locale: BCP-47 ("es", "es-ES", "en-US", "auto"). Defaults to es-ES.
//
// Protocol: a single JSON object on stdout.
//   success: {"ok":true,"text":"...","latency_s":0.23,"locale":"es-ES"}
//   failure: {"ok":false,"error":"..."}
// Stage logs go to stderr (ignored by the caller, useful for debugging).
//
// Notes (hard-won — see project memory):
//  * SpeechAnalyzer on-device does NOT need SFSpeechRecognizer.requestAuthorization
//    (that call hangs in a CLI without an Info.plist). Don't call it.
//  * Top-level code runs on the MainActor; never block it with a semaphore around
//    a Task — use top-level await directly.

struct Output: Codable {
    var ok: Bool
    var text: String?
    var latency_s: Double?
    var locale: String?
    var error: String?
}

func logErr(_ s: String) {
    FileHandle.standardError.write(("[asr] " + s + "\n").data(using: .utf8)!)
}

func emitJSON(_ o: Output) {
    let enc = JSONEncoder()
    if let data = try? enc.encode(o), let s = String(data: data, encoding: .utf8) {
        print(s)
    } else {
        print("{\"ok\":false,\"error\":\"json encode failed\"}")
    }
    fflush(stdout)
}

func fail(_ msg: String) -> Never {
    emitJSON(Output(ok: false, text: nil, latency_s: nil, locale: nil, error: msg))
    exit(1)
}

// Resolve the requested locale against what SpeechTranscriber actually supports.
// Exact match wins; otherwise fall back to any supported locale sharing the
// language code (e.g. "es" or "es-CO" → "es-ES"); otherwise nil.
// Canonical region per language so "es"/"auto" land on es-ES (not es-CL by
// alphabetical luck) and "en" on en-US.
let preferredByLang = ["es": "es-ES", "en": "en-US", "pt": "pt-BR",
                       "fr": "fr-FR", "de": "de-DE", "it": "it-IT", "zh": "zh-CN"]

@available(macOS 26.0, *)
func bestForLanguage(_ langCode: String, in supported: [Locale]) -> Locale? {
    let candidates = supported.filter { $0.language.languageCode?.identifier == langCode }
    if candidates.isEmpty { return nil }
    if let pref = preferredByLang[langCode]?.lowercased(),
       let m = candidates.first(where: { $0.identifier(.bcp47).lowercased() == pref }) {
        return m
    }
    // Deterministic fallback: lowest bcp47 id.
    return candidates.min(by: { $0.identifier(.bcp47) < $1.identifier(.bcp47) })
}

@available(macOS 26.0, *)
func resolveLocale(_ requested: String) async -> Locale? {
    let supported = Array(await SpeechTranscriber.supportedLocales)
    let req = requested.lowercased()

    if req == "auto" {
        // Prefer the system locale if supported, else canonical Spanish, else first.
        let sys = Locale.current.identifier(.bcp47).lowercased()
        if let m = supported.first(where: { $0.identifier(.bcp47).lowercased() == sys }) { return m }
        if let langCode = Locale.current.language.languageCode?.identifier,
           let m = bestForLanguage(langCode, in: supported) { return m }
        return bestForLanguage("es", in: supported) ?? supported.first
    }

    if let exact = supported.first(where: { $0.identifier(.bcp47).lowercased() == req }) {
        return exact
    }
    // Match by language code prefix, preferring the canonical region.
    let langCode = String(req.prefix(2))
    return bestForLanguage(langCode, in: supported)
}

@available(macOS 26.0, *)
func transcribe(path: String, requestedLocale: String) async throws -> Output {
    guard let locale = await resolveLocale(requestedLocale) else {
        return Output(ok: false, text: nil, latency_s: nil, locale: nil,
                      error: "locale '\(requestedLocale)' not supported by SpeechTranscriber")
    }
    let bcp47 = locale.identifier(.bcp47)
    logErr("resolved locale \(requestedLocale) → \(bcp47)")

    let transcriber = SpeechTranscriber(
        locale: locale,
        transcriptionOptions: [],
        reportingOptions: [],
        attributeOptions: []
    )

    // Ensure on-device language assets are installed (one-time download).
    if let req = try await AssetInventory.assetInstallationRequest(supporting: [transcriber]) {
        logErr("downloading language assets for \(bcp47)…")
        try await req.downloadAndInstall()
        logErr("assets installed")
    }

    let analyzer = SpeechAnalyzer(modules: [transcriber])
    let audioFile = try AVAudioFile(forReading: URL(fileURLWithPath: path))

    let t0 = ContinuousClock.now
    let resultsTask = Task {
        var acc = AttributedString()
        for try await result in transcriber.results {
            acc += result.text
        }
        return acc
    }

    if let last = try await analyzer.analyzeSequence(from: audioFile) {
        try await analyzer.finalizeAndFinish(through: last)
    } else {
        try await analyzer.finalizeAndFinishThroughEndOfInput()
    }

    let finalText = try await resultsTask.value
    let dur = t0.duration(to: .now)
    let seconds = Double(dur.components.seconds) + Double(dur.components.attoseconds) / 1e18

    let text = String(finalText.characters).trimmingCharacters(in: .whitespacesAndNewlines)
    return Output(ok: true, text: text, latency_s: seconds, locale: bcp47, error: nil)
}

// ---- entry point (top-level await; no Task/semaphore) ----

let args = CommandLine.arguments
guard args.count >= 2 else {
    fail("usage: asr <path-to-audio> [locale]")
}
let audioPath = args[1]
let requestedLocale = args.count >= 3 ? args[2] : "es-ES"

guard FileManager.default.fileExists(atPath: audioPath) else {
    fail("audio file not found: \(audioPath)")
}

if #available(macOS 26.0, *) {
    do {
        let out = try await transcribe(path: audioPath, requestedLocale: requestedLocale)
        emitJSON(out)
        exit(out.ok ? 0 : 1)
    } catch {
        fail("\(error)")
    }
} else {
    fail("SpeechAnalyzer requires macOS 26+")
}
