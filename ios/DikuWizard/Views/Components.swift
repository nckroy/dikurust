import SwiftUI

/// Shows `content` once you're logged in as an immortal; otherwise explains
/// what's needed and offers to connect.
struct WizardGate<Content: View>: View {
    @Environment(GameSession.self) private var session
    @Environment(AppSettings.self) private var settings
    var minimumLevel = 21
    @ViewBuilder var content: (Snapshot) -> Content

    var body: some View {
        if let snap = session.snapshot, snap.viewerLevel >= minimumLevel, session.isPlaying {
            content(snap)
        } else {
            gate
        }
    }

    @ViewBuilder private var gate: some View {
        switch session.phase {
        case .connecting, .loggingIn:
            ProgressView("Logging in as \(session.characterName)…")
        case .playing where session.snapshotError != nil:
            ContentUnavailableView {
                Label("Immortals only", systemImage: "lock")
            } description: {
                Text("\(session.characterName) is logged in but isn't level 21 or above, so the server won't share the wizard view. The console still works.")
            }
        case .playing where session.snapshot != nil:
            ContentUnavailableView {
                Label("Implementors only", systemImage: "lock")
            } description: {
                Text("This needs level \(minimumLevel). \(session.characterName) is level \(session.viewerLevel).")
            }
        case .playing:
            ProgressView("Asking the server…")
        case .failed(let why):
            ContentUnavailableView {
                Label("Couldn't log in", systemImage: "exclamationmark.triangle")
            } description: {
                Text(why)
            } actions: {
                Button("Try again") { session.connect(settings: settings) }
                    .buttonStyle(.borderedProminent)
            }
        case .disconnected:
            ContentUnavailableView {
                Label("Not connected", systemImage: "wand.and.stars")
            } description: {
                Text(settings.hasCredentials
                     ? "Log in as \(settings.character) to see who's on and work your magic."
                     : "Add your immortal character's name and password in Settings.")
            } actions: {
                if settings.hasCredentials {
                    Button("Connect") { session.connect(settings: settings) }
                        .buttonStyle(.borderedProminent)
                }
            }
        }
    }
}

/// What a command printed, in the game's monospaced font.
struct CommandResult: Identifiable {
    let id = UUID()
    let command: String
    let output: String
}

struct CommandResultSheet: View {
    let result: CommandResult
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView([.vertical]) {
                Text(result.output.isEmpty ? "(no output)" : result.output)
                    .font(.system(.footnote, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
            }
            .navigationTitle(result.command)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) { Button("Done") { dismiss() } }
                ToolbarItem(placement: .topBarLeading) {
                    ShareLink(item: result.output) { Image(systemName: "square.and.arrow.up") }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }
}

/// Runs a game command and shows what came back.
@MainActor
@Observable
final class CommandRunner {
    var result: CommandResult?
    var running: String?

    func run(_ command: String, on session: GameSession, showResult: Bool = true) {
        guard running == nil else { return }
        running = command
        Task {
            let out = await session.run(command)
            running = nil
            if showResult { result = CommandResult(command: command, output: out) }
        }
    }
}

struct StatTile: View {
    let title: String
    let value: String
    var symbol: String? = nil
    var tint: Color = .accentColor

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                if let symbol { Image(systemName: symbol).foregroundStyle(tint) }
                Text(title).font(.caption).foregroundStyle(.secondary)
            }
            Text(value).font(.title2.weight(.semibold)).monospacedDigit()
                .lineLimit(1).minimumScaleFactor(0.6)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(.background.secondary, in: RoundedRectangle(cornerRadius: 12))
    }
}

struct PointsBar: View {
    let label: String
    let value: Int
    let max: Int
    let color: Color

    var body: some View {
        HStack {
            Text(label).font(.caption).frame(width: 44, alignment: .leading).foregroundStyle(.secondary)
            ProgressView(value: Double(Swift.max(0, Swift.min(value, max))), total: Double(Swift.max(max, 1)))
                .tint(color)
            Text("\(value)/\(max)").font(.caption.monospacedDigit()).frame(minWidth: 64, alignment: .trailing)
        }
    }
}

/// A one-field prompt (vnum, message, …) that then runs a command.
struct PromptAction: Identifiable {
    let id = UUID()
    let title: String
    let message: String
    let placeholder: String
    var numeric = false
    let make: (String) -> String
}

extension View {
    func promptAction(_ action: Binding<PromptAction?>, perform: @escaping (String) -> Void) -> some View {
        modifier(PromptActionModifier(action: action, perform: perform))
    }
}

private struct PromptActionModifier: ViewModifier {
    @Binding var action: PromptAction?
    let perform: (String) -> Void
    @State private var text = ""

    func body(content: Content) -> some View {
        content.alert(action?.title ?? "", isPresented: Binding(
            get: { action != nil },
            set: { if !$0 { action = nil } }
        ), presenting: action) { a in
            TextField(a.placeholder, text: $text)
                .keyboardType(a.numeric ? .numberPad : .default)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
            Button("Cancel", role: .cancel) { text = "" }
            Button("Run") {
                let v = text.trimmingCharacters(in: .whitespaces)
                text = ""
                if !v.isEmpty { perform(a.make(v)) }
            }
        } message: { a in
            Text(a.message)
        }
    }
}
