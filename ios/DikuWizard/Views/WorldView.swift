import SwiftUI

struct WorldView: View {
    @Environment(GameSession.self) private var session
    @State private var runner = CommandRunner()
    @State private var prompt: PromptAction?
    @State private var confirmPurgeRoom = false

    var body: some View {
        WizardGate { snap in
            List {
                SwiftUI.Section("Here") {
                    tool("Look", "eye", "look")
                    tool("Stat this room", "info.circle", "stat room")
                    ask("Go to room…", "figure.walk.arrival",
                        PromptAction(title: "Go to", message: "A room number, or the name of a character or object.", placeholder: "3001") { "goto \($0)" })
                    ask("Look somewhere else…", "binoculars",
                        PromptAction(title: "Look at", message: "Runs “look” in that room without moving you.", placeholder: "3014") { "at \($0) look" })
                    ask("Where is…", "location.magnifyingglass",
                        PromptAction(title: "Where", message: "Searches the world for characters and objects.", placeholder: "fido") { "where \($0)" })
                }
                SwiftUI.Section {
                    NavigationLink { CatalogView() } label: {
                        Label("Browse mobiles, objects and rooms", systemImage: "books.vertical")
                    }
                }
                if snap.viewerLevel >= 22 {
                    SwiftUI.Section("Create & destroy") {
                        ask("Load a mobile…", "pawprint",
                            PromptAction(title: "Load mobile", message: "Mobile prototype number. It appears in your room.", placeholder: "3062", numeric: true) { "load char \($0)" })
                        ask("Load an object…", "shippingbox",
                            PromptAction(title: "Load object", message: "Object prototype number. It appears in your room.", placeholder: "3022", numeric: true) { "load obj \($0)" })
                        Button(role: .destructive) { confirmPurgeRoom = true } label: {
                            Label("Purge this room", systemImage: "trash")
                        }
                    }
                }
                SwiftUI.Section("Speak") {
                    ask("Wiz channel…", "sparkles",
                        PromptAction(title: "Wiz channel", message: "To every immortal online.", placeholder: "Message") { "wiz \($0)" })
                    ask("Echo to the room…", "text.bubble",
                        PromptAction(title: "Echo", message: "Raw text, no name attached, to everyone else in your room.", placeholder: "The ground trembles.") { "echo \($0)" })
                    ask("Shout…", "megaphone",
                        PromptAction(title: "Shout", message: "To everyone in the game who hears shouts.", placeholder: "Message") { "shout \($0)" })
                }
                SwiftUI.Section("Zones (\(snap.zones.count))") {
                    ForEach(snap.zones) { z in ZoneRow(zone: z) }
                }
                SwiftUI.Section("Database") {
                    LabeledContent("Rooms", value: "\(snap.counts.rooms)")
                    LabeledContent("Mobiles alive / prototypes", value: "\(snap.counts.mobs) / \(snap.counts.mobProtos)")
                    LabeledContent("Objects / prototypes", value: "\(snap.counts.objects) / \(snap.counts.objProtos)")
                    LabeledContent("Player files", value: "\(snap.counts.playerFiles)")
                }
            }
            .refreshable { session.requestSnapshot() }
            .confirmationDialog("Purge this room?", isPresented: $confirmPurgeRoom, titleVisibility: .visible) {
                Button("Destroy every mobile and object here", role: .destructive) { runner.run("purge", on: session) }
            } message: {
                Text("Players are left alone.")
            }
        }
        .navigationTitle("World")
        .sheet(item: $runner.result) { CommandResultSheet(result: $0) }
        .promptAction($prompt) { runner.run($0, on: session) }
    }

    private func tool(_ title: String, _ symbol: String, _ command: String) -> some View {
        Button { runner.run(command, on: session) } label: { Label(title, systemImage: symbol) }
            .disabled(runner.running != nil)
    }

    private func ask(_ title: String, _ symbol: String, _ action: PromptAction) -> some View {
        Button { prompt = action } label: { Label(title, systemImage: symbol) }
            .disabled(runner.running != nil)
    }
}

struct ZoneRow: View {
    let zone: Zone

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(zone.name).font(.headline).lineLimit(1)
                Spacer()
                if zone.players > 0 {
                    Label("\(zone.players)", systemImage: "person.fill").font(.caption).foregroundStyle(.green)
                }
                Label("\(zone.mobs)", systemImage: "pawprint.fill").font(.caption).foregroundStyle(.secondary)
            }
            HStack {
                Text(verbatim: "Rooms \(zone.bottom)–\(zone.top)").font(.caption.monospacedDigit()).foregroundStyle(.secondary)
                Spacer()
                Text(zone.resetMode == 0 ? "never resets" : "reset in \(max(0, zone.lifespan - zone.age)) min")
                    .font(.caption).foregroundStyle(.secondary)
            }
            if zone.resetMode != 0 {
                ProgressView(value: Double(min(zone.age, zone.lifespan)), total: Double(max(zone.lifespan, 1)))
                    .tint(zone.age >= zone.lifespan ? .orange : .accentColor)
            }
        }
        .padding(.vertical, 2)
        .accessibilityElement(children: .combine)
        .accessibilityHint(zone.resetModeDescription)
    }
}
