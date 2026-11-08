import Foundation
import Observation

/// A logged-in game connection over `/ws`, the same socket the web client
/// uses. Logs in as your character, keeps a transcript, runs commands and
/// asks for the immortal-only snapshot (`\u{1}snap`).
///
/// Every wizard action in the app is an ordinary game command sent here, so
/// the server's own level checks and logging apply exactly as if you had
/// typed it.
@MainActor
@Observable
final class GameSession {
    enum Phase: Equatable {
        case disconnected
        case connecting
        case loggingIn
        case playing
        case failed(String)

        var isActive: Bool { self == .connecting || self == .loggingIn || self == .playing }
    }

    struct Line: Identifiable, Equatable {
        enum Kind { case output, input, system }
        let id: Int
        let kind: Kind
        let text: String
    }

    private(set) var phase: Phase = .disconnected
    private(set) var lines: [Line] = []
    private(set) var snapshot: Snapshot?
    private(set) var snapshotUpdated: Date?
    /// Set when the server refuses the snapshot (you aren't immortal).
    private(set) var snapshotError: String?
    /// Every prototype, fetched once per login when a view asks for it.
    private(set) var catalog: Catalog?
    private(set) var catalogError: String?
    private(set) var catalogLoading = false
    /// False while the server has asked to hide input (password prompts).
    private(set) var echo = true
    private(set) var characterName = ""

    var isPlaying: Bool { phase == .playing }
    var isImmortal: Bool { snapshot != nil }
    var viewerLevel: Int { snapshot?.viewerLevel ?? 0 }

    private var task: URLSessionWebSocketTask?
    private var receiveTask: Task<Void, Never>?
    private var snapshotTask: Task<Void, Never>?
    private var nextLineID = 0
    private var password = ""
    private var passwordSent = false
    private var loginTail = ""
    private var capture: (buffer: String, done: CheckedContinuation<String, Never>, pages: Int)?
    private static let maxLines = 3000

    // MARK: connecting

    func connect(settings: AppSettings) {
        guard !phase.isActive else { return }
        guard let url = settings.webSocketURL else {
            phase = .failed("The server address isn't a valid URL.")
            return
        }
        guard settings.hasCredentials else {
            phase = .failed("Add your character's name and password in Settings first.")
            return
        }
        characterName = settings.character.trimmingCharacters(in: .whitespaces)
        password = settings.password
        passwordSent = false
        loginTail = ""
        snapshot = nil
        snapshotError = nil
        catalog = nil
        catalogError = nil
        catalogLoading = false
        echo = true
        phase = .connecting
        system("Connecting to \(url.absoluteString) as \(characterName)…")

        let t = URLSession.shared.webSocketTask(with: url)
        t.maximumMessageSize = 4 * 1024 * 1024
        task = t
        t.resume()
        receiveTask = Task { [weak self] in await self?.receiveLoop(t) }
    }

    /// `quit`: saves the character and leaves the game cleanly.
    func quit() {
        Task { await leave() }
    }

    /// Sends `quit`, gives the server a moment to save the character and
    /// close, then drops the link. Not playing: just drops the link.
    func leave() async {
        guard phase == .playing else { return disconnect() }
        send("quit")
        try? await Task.sleep(for: .seconds(1.5))
        disconnect()
    }

    /// Drops the link. The character stays in the game, link-dead, until
    /// you reconnect.
    func disconnect() {
        snapshotTask?.cancel()
        receiveTask?.cancel()
        task?.cancel(with: .normalClosure, reason: nil)
        task = nil
        catalogLoading = false
        if phase.isActive { system("Disconnected.") }
        if case .failed = phase {} else { phase = .disconnected }
        finishCapture()
    }

    // MARK: sending

    /// Send one line of input, as if typed.
    func send(_ line: String, echoInTranscript: Bool = true) {
        guard let task else { return }
        if echoInTranscript { append(.input, echo ? line : String(repeating: "•", count: line.count)) }
        task.send(.string(line)) { _ in }
    }

    /// Send a command and collect what the game says back until the next
    /// prompt (following any "Press return" pages). Times out after a few
    /// seconds with whatever arrived.
    func run(_ command: String) async -> String {
        while capture != nil { try? await Task.sleep(for: .milliseconds(100)) }
        guard isPlaying else { return "Not connected." }
        let result = await withCheckedContinuation { (c: CheckedContinuation<String, Never>) in
            capture = ("", c, 0)
            send(command)
            Task {
                try? await Task.sleep(for: .seconds(4))
                self.finishCapture()
            }
        }
        requestSnapshot(after: .milliseconds(300))
        return result
    }

    func requestSnapshot(after delay: Duration = .zero) {
        Task {
            if delay > .zero { try? await Task.sleep(for: delay) }
            guard self.isPlaying, let task = self.task else { return }
            task.send(.string("\u{1}snap")) { _ in }
        }
    }

    /// Ask for the prototype catalog. It is large and changes only with a
    /// new world, so it is fetched once per login unless `force` is set.
    func requestCatalog(force: Bool = false) {
        guard isPlaying, let task, !catalogLoading else { return }
        guard force || catalog == nil else { return }
        catalogLoading = true
        catalogError = nil
        task.send(.string("\u{1}catalog")) { _ in }
    }

    func clearTranscript() { lines.removeAll() }

    // MARK: receiving

    private func receiveLoop(_ t: URLSessionWebSocketTask) async {
        while !Task.isCancelled {
            do {
                let msg = try await t.receive()
                switch msg {
                case .string(let s): handle(frame: s)
                case .data(let d): handle(frame: String(decoding: d, as: UTF8.self))
                @unknown default: break
                }
            } catch {
                guard t === task else { return }
                connectionEnded(error, task: t)
                return
            }
        }
    }

    private func connectionEnded(_ error: Error, task t: URLSessionWebSocketTask) {
        snapshotTask?.cancel()
        catalogLoading = false
        let code = (t.response as? HTTPURLResponse)?.statusCode
        let message: String
        switch (phase, code) {
        case (_, 503?):
            message = "The server refused the connection. It is wizlocked, or full."
        case (.connecting, _):
            message = "Couldn't connect: \(error.localizedDescription)"
        case (.playing, _):
            message = "The connection closed."
        default:
            message = "The connection closed during login."
        }
        task = nil
        finishCapture()
        if case .failed = phase {
            // keep the more specific login error
        } else if phase == .playing {
            phase = .disconnected
            system(message)
        } else {
            phase = .failed(message)
            system(message)
        }
    }

    private struct Frame: Decodable {
        var t: String
        var d: String?
        var v: Bool?
        var err: String?
    }

    private struct WizFrame: Decodable {
        var d: Snapshot?
        var err: String?
    }

    private struct CatalogFrame: Decodable {
        var d: Catalog?
        var err: String?
    }

    private struct FrameType: Decodable {
        var t: String
    }

    private func handle(frame: String) {
        let data = Data(frame.utf8)
        // "wiz" and "cat" carry an object in "d", which Frame can't decode.
        switch (try? JSONDecoder().decode(FrameType.self, from: data))?.t {
        case "wiz":
            guard let w = try? JSONDecoder.diku.decode(WizFrame.self, from: data) else { return }
            // A server from before the catalog answers `catalog` this way.
            // That says nothing about the snapshot, so don't treat it as a refusal.
            if w.d == nil, w.err == "Unknown request.", catalogLoading {
                catalogLoading = false
                catalogError = "This server is too old to send a catalog. It needs the build with the catalog request (src/admin.rs)."
                return
            }
            handleWiz(w)
            return
        case "cat":
            catalogLoading = false
            do {
                let c = try JSONDecoder.diku.decode(CatalogFrame.self, from: data)
                if let d = c.d { catalog = d } else { catalogError = c.err ?? "The server sent an empty catalog." }
            } catch {
                catalogError = "Couldn't read the catalog: \(error.localizedDescription)"
            }
            return
        default:
            break
        }
        guard let f = try? JSONDecoder().decode(Frame.self, from: data) else { return }
        switch f.t {
        case "o":
            output(Self.clean(f.d ?? ""))
        case "echo":
            echo = f.v ?? true
        default:
            break
        }
    }

    private func handleWiz(_ w: WizFrame) {
        if let s = w.d {
            snapshot = s
            snapshotUpdated = Date()
            snapshotError = nil
        } else if let e = w.err {
            snapshotError = e
            snapshotTask?.cancel()
        }
    }

    private static func clean(_ s: String) -> String {
        s.replacingOccurrences(of: "\n\r", with: "\n").replacingOccurrences(of: "\r", with: "")
    }

    private func output(_ text: String) {
        append(.output, text)
        if phase == .connecting || phase == .loggingIn {
            login(text)
        }
        if var c = capture {
            c.buffer += text
            capture = c
            let tail = text.trimmingCharacters(in: .whitespaces)
            if tail.hasSuffix("*** Press return ***"), c.pages < 20 {
                capture?.pages += 1
                send("", echoInTranscript: false)
            } else if text.hasSuffix("> ") {
                finishCapture()
            }
        }
    }

    private func finishCapture() {
        guard let c = capture else { return }
        capture = nil
        var out = c.buffer
        if out.hasSuffix("> ") { out.removeLast(2) }
        out = out.replacingOccurrences(of: "*** Press return ***", with: "")
        c.done.resume(returning: out.trimmingCharacters(in: .newlines))
    }

    // MARK: the login conversation (nanny)

    private func login(_ text: String) {
        phase = .loggingIn
        // Match against everything since the last prompt we answered, not
        // just a short window: entering the game sends the welcome line and
        // the whole room description in one frame, which can push the
        // welcome far from the end.
        let tail = loginTail + text
        loginTail = String(tail.suffix(400))

        func fail(_ why: String) {
            phase = .failed(why)
            system(why)
            loginTail = ""
            Task {
                try? await Task.sleep(for: .milliseconds(300))
                self.disconnect()
            }
        }

        if tail.contains("Already playing, cannot connect") {
            return fail("\(characterName) is already playing from somewhere else. Quit there (or let that link drop) and try again.")
        }
        if tail.contains("Wrong password") {
            return fail("Wrong password for \(characterName).")
        }
        if tail.contains("Illegal name") {
            return fail("\"\(characterName)\" isn't a legal character name.")
        }
        if tail.contains("Did I get that right") {
            send("n", echoInTranscript: false)
            return fail("There is no character called \(characterName) on this server. Create one in the web client first.")
        }
        if tail.contains("Welcome to the land of DikuMUD") || tail.contains("Reconnecting.") {
            loginTail = ""
            phase = .playing
            system("Logged in as \(characterName).")
            startSnapshots()
            return
        }
        if tail.hasSuffix("known? ") || tail.hasSuffix("Name: ") {
            loginTail = ""
            send(characterName)
        } else if tail.hasSuffix("Password: ") {
            loginTail = ""
            guard !passwordSent else { return fail("The server asked for the password again.") }
            passwordSent = true
            send(password, echoInTranscript: false)
            append(.input, "••••••••")
        } else if tail.contains("*** PRESS RETURN") {
            loginTail = ""
            send("")
        } else if tail.hasSuffix("Make your choice: ") {
            loginTail = ""
            send("1")
        }
    }

    private func startSnapshots() {
        snapshotTask?.cancel()
        snapshotTask = Task { [weak self] in
            while !Task.isCancelled {
                guard let self, self.isPlaying, self.snapshotError == nil else { return }
                self.requestSnapshot()
                try? await Task.sleep(for: .seconds(5))
            }
        }
    }

    // MARK: transcript

    private func system(_ text: String) { append(.system, text) }

    private func append(_ kind: Line.Kind, _ text: String) {
        nextLineID += 1
        lines.append(Line(id: nextLineID, kind: kind, text: text))
        if lines.count > Self.maxLines { lines.removeFirst(lines.count - Self.maxLines) }
    }
}
