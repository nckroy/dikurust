import SwiftUI

/// Game-wide switches: wizlock, shutdown, reboot.
struct ServerControlView: View {
    @Environment(GameSession.self) private var session
    @State private var runner = CommandRunner()
    @State private var confirm: Danger?
    @State private var typed = ""

    enum Danger: String, Identifiable {
        case wizlock, unlock, shutdown, reboot
        var id: String { rawValue }
        var command: String {
            switch self {
            case .wizlock, .unlock: return "wizlock"
            case .shutdown: return "shutdown"
            case .reboot: return "shutdown reboot"
            }
        }
    }

    var body: some View {
        WizardGate { snap in
            List {
                SwiftUI.Section {
                    LabeledContent("Uptime", value: formatUptime(snap.server.uptime))
                    LabeledContent("Playing", value: "\(snap.server.playing)")
                    LabeledContent("Connections", value: "\(snap.server.connections)")
                    LabeledContent("Version", value: snap.server.version)
                }
                SwiftUI.Section {
                    Button { runner.run("save", on: session) } label: { Label("Save my character", systemImage: "square.and.arrow.down") }
                    if snap.viewerLevel >= 22 {
                        Button { runner.run("transfer all", on: session) } label: { Label("Bring everyone to me", systemImage: "person.3.sequence") }
                    }
                }
                if snap.viewerLevel >= 24 {
                    SwiftUI.Section {
                        Button { confirm = snap.server.wizlock ? .unlock : .wizlock } label: {
                            Label(snap.server.wizlock ? "Lift the wizlock" : "Wizlock the game",
                                  systemImage: snap.server.wizlock ? "lock.open" : "lock")
                        }
                    } header: {
                        Text("Wizlock")
                    } footer: {
                        Text(snap.server.wizlock
                             ? "The game is refusing every new connection, including yours. Lift it before you disconnect, or restart the server to clear it."
                             : "Refuses every new connection, including immortals and this app. Only someone still connected can lift it.")
                    }
                    SwiftUI.Section {
                        Button(role: .destructive) { confirm = .reboot } label: { Label("Reboot", systemImage: "arrow.clockwise.circle") }
                        Button(role: .destructive) { confirm = .shutdown } label: { Label("Shut down", systemImage: "power") }
                    } header: {
                        Text("Stop the game")
                    } footer: {
                        Text("Both announce it, save everyone online and stop the server. On diku.earth the container restarts on its own, so either one brings the world back in a few seconds.")
                    }
                } else {
                    SwiftUI.Section {
                        Text("Wizlock, shutdown and reboot need level 24.").foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle("Server")
        .sheet(item: $runner.result) { CommandResultSheet(result: $0) }
        .alert(title(confirm), isPresented: Binding(get: { confirm != nil }, set: { if !$0 { confirm = nil; typed = "" } }), presenting: confirm) { d in
            if d == .shutdown || d == .reboot {
                TextField("Type \(d.rawValue)", text: $typed)
                    .textInputAutocapitalization(.never).autocorrectionDisabled()
            }
            Button("Cancel", role: .cancel) { typed = "" }
            Button(d == .unlock ? "Lift it" : d.rawValue.capitalized, role: d == .unlock ? nil : .destructive) {
                let ok = (d != .shutdown && d != .reboot) || typed.lowercased() == d.rawValue
                typed = ""
                if ok { runner.run(d.command, on: session) }
            }
        } message: { d in
            Text(message(d))
        }
    }

    private func title(_ d: Danger?) -> String {
        switch d {
        case .wizlock: return "Wizlock the game?"
        case .unlock: return "Lift the wizlock?"
        case .shutdown: return "Shut down the game?"
        case .reboot: return "Reboot the game?"
        case nil: return ""
        }
    }

    private func message(_ d: Danger) -> String {
        switch d {
        case .wizlock: return "Nobody, including you, can connect until someone still online lifts it or the server restarts."
        case .unlock: return "New connections will be allowed again."
        case .shutdown, .reboot: return "Everyone online is saved and disconnected. Type “\(d.rawValue)” to confirm."
        }
    }
}
