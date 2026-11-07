import SwiftUI

/// Everything you can do to or for one player.
struct PlayerDetailView: View {
    let playerID: String
    @Environment(GameSession.self) private var session
    @State private var runner = CommandRunner()
    @State private var prompt: PromptAction?
    @State private var confirmPurge = false
    @State private var advanceLevel = 21

    var body: some View {
        if let snap = session.snapshot, let p = snap.players.first(where: { $0.id == playerID }) {
            content(p, viewer: snap.viewerLevel)
                .navigationTitle(p.displayName)
                .sheet(item: $runner.result) { CommandResultSheet(result: $0) }
                .promptAction($prompt) { runner.run($0, on: session) }
                .overlay(alignment: .bottom) {
                    if let r = runner.running {
                        Label("Running \(r)…", systemImage: "hourglass")
                            .padding(10).background(.thinMaterial, in: Capsule()).padding()
                    }
                }
        } else {
            ContentUnavailableView("Gone", systemImage: "person.slash",
                                   description: Text("That player has left the game."))
        }
    }

    @ViewBuilder
    private func content(_ p: Player, viewer: Int) -> some View {
        let name = p.name ?? ""
        List {
            SwiftUI.Section {
                if let t = p.title, !t.isEmpty { Text("\(name) \(t)").font(.headline) }
                LabeledContent("Level", value: p.level.map { "\($0) · \(Rank.name(for: $0))" } ?? "–")
                if let c = p.class { LabeledContent("Class", value: c) }
                LabeledContent("Connection", value: p.linkdead ? "link-dead" : p.state)
                if !p.host.isEmpty { LabeledContent("Address", value: p.host).textSelection(.enabled) }
                if let r = p.roomName, !r.isEmpty { LabeledContent("Room", value: "\(r) [\(String(p.room ?? -1))]") }
                if let pos = p.position { LabeledContent("Position", value: pos) }
                if let idle = p.idleDescription { LabeledContent("Idle", value: idle) }
                if let s = p.switchedInto { LabeledContent("Switched into", value: s) }
            }
            if let hit = p.hit, let mh = p.maxHit {
                SwiftUI.Section("Condition") {
                    PointsBar(label: "HP", value: hit, max: mh, color: .red)
                    if let m = p.mana, let mm = p.maxMana { PointsBar(label: "Mana", value: m, max: mm, color: .blue) }
                    if let v = p.move, let mv = p.maxMove { PointsBar(label: "Move", value: v, max: mv, color: .green) }
                    if let g = p.gold { LabeledContent("Gold", value: g.formatted()) }
                    if let x = p.exp { LabeledContent("Experience", value: x.formatted()) }
                    if let a = p.alignment { LabeledContent("Alignment", value: "\(a)") }
                }
            }

            if !name.isEmpty && (p.isPlaying || p.linkdead) {
                SwiftUI.Section("Look") {
                    action("Stat", "info.circle", "stat \(name)")
                    action("Where", "location.magnifyingglass", "where \(name)")
                    action("Look at them", "eye", "at \(name) look")
                }
                SwiftUI.Section("Move") {
                    action("Go to them", "figure.walk.arrival", "goto \(name)")
                    if viewer >= 22 { action("Bring them here", "arrow.down.to.line", "transfer \(name)") }
                }
                SwiftUI.Section("Help & talk") {
                    if viewer >= 22 { action("Restore", "cross.case", "restore \(name)") }
                    Button { prompt = PromptAction(title: "Tell \(name)", message: "Sends a private tell.", placeholder: "Message") { "tell \(name) \($0)" } } label: {
                        Label("Tell…", systemImage: "bubble.left")
                    }
                    if viewer >= 22 {
                        Button { prompt = PromptAction(title: "Force \(name)", message: "They see “\(session.characterName) has forced you to …” and run the command.", placeholder: "Command") { "force \(name) \($0)" } } label: {
                            Label("Force…", systemImage: "hand.point.right")
                        }
                    }
                    if (p.level ?? 99) < viewer {
                        action(p.snooping == true ? "Snoop (already snooping someone)" : "Snoop", "binoculars", "snoop \(name)")
                    }
                }
                if viewer >= 22, (p.level ?? 99) <= viewer {
                    SwiftUI.Section {
                        flagToggle("Can't shout", p.noshout, "noshout \(name)")
                        flagToggle("Can't tell", p.notell, "notell \(name)")
                        flagToggle("Can't emote", p.noemote, "noemote \(name)")
                        if viewer >= 23 { flagToggle("Frozen", p.frozen, "freeze \(name)") }
                    } header: {
                        Text("Moderation")
                    } footer: {
                        Text("These are saved with the character.")
                    }
                }
                if viewer >= 24, (p.level ?? 0) < 24, p.isPlaying {
                    SwiftUI.Section {
                        Stepper("Advance to level \(advanceLevel)", value: $advanceLevel, in: max(21, (p.level ?? 0) + 1)...24)
                        action("Advance", "arrow.up.circle", "advance \(name) \(advanceLevel)")
                    } header: {
                        Text("Promote")
                    } footer: {
                        Text("“advance” only works when they're in the same room as you. Go to them first.")
                    }
                }
                if viewer >= 22, (p.level ?? 99) < viewer {
                    SwiftUI.Section {
                        Button(role: .destructive) { confirmPurge = true } label: {
                            Label("Purge \(name)", systemImage: "trash")
                        }
                    } footer: {
                        Text("Destroys the character in the game and closes their connection without saving.")
                    }
                    .confirmationDialog("Purge \(name)?", isPresented: $confirmPurge, titleVisibility: .visible) {
                        Button("Purge without saving", role: .destructive) { runner.run("purge \(name)", on: session) }
                    } message: {
                        Text("They lose everything since their last save, and what they carried drops to the floor. Level 22 can only purge someone in the same room.")
                    }
                }
            }
        }
        .refreshable { session.requestSnapshot() }
    }

    private func action(_ title: String, _ symbol: String, _ command: String) -> some View {
        Button { runner.run(command, on: session) } label: {
            HStack {
                Label(title, systemImage: symbol)
                Spacer()
                Text(command).font(.caption.monospaced()).foregroundStyle(.tertiary).lineLimit(1)
            }
        }
        .disabled(runner.running != nil)
    }

    /// A flag the server toggles: shows the server's current state, and a
    /// tap sends the toggling command. No optimistic flip: the row changes
    /// when the next snapshot says it did.
    private func flagToggle(_ title: String, _ value: Bool?, _ command: String) -> some View {
        let on = value ?? false
        return Button { runner.run(command, on: session, showResult: false) } label: {
            HStack {
                Text(title).foregroundStyle(.primary)
                Spacer()
                if runner.running == command {
                    ProgressView()
                } else {
                    Text(on ? "On" : "Off")
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(on ? Color.orange : Color.secondary)
                    Image(systemName: on ? "checkmark.circle.fill" : "circle")
                        .foregroundStyle(on ? Color.orange : Color.secondary)
                }
            }
        }
        .disabled(runner.running != nil)
        .accessibilityValue(on ? "On" : "Off")
        .accessibilityHint("Sends \(command)")
    }
}
