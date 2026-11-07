import SwiftUI

struct PlayersView: View {
    @Environment(GameSession.self) private var session
    @State private var search = ""

    var body: some View {
        WizardGate { snap in
            let players = filtered(snap.players)
            List {
                group("Immortals", players.filter { $0.isPlaying && $0.isImmortal })
                group("Mortals", players.filter { $0.isPlaying && !$0.isImmortal })
                group("Link-dead", players.filter { $0.linkdead })
                group("Logging in", players.filter { !$0.isPlaying && !$0.linkdead })
            }
            .overlay {
                if snap.players.isEmpty {
                    ContentUnavailableView("Nobody's here", systemImage: "person.slash")
                }
            }
            .searchable(text: $search, prompt: "Name, room or address")
            .refreshable { session.requestSnapshot() }
        }
        .navigationTitle("Players")
        .navigationDestination(for: String.self) { id in
            PlayerDetailView(playerID: id)
        }
    }

    private func filtered(_ ps: [Player]) -> [Player] {
        let q = search.trimmingCharacters(in: .whitespaces).lowercased()
        guard !q.isEmpty else { return ps }
        return ps.filter {
            ($0.name ?? "").lowercased().contains(q)
                || ($0.roomName ?? "").lowercased().contains(q)
                || $0.host.contains(q)
                || ($0.room.map(String.init) ?? "") == q
        }
    }

    @ViewBuilder private func group(_ title: String, _ ps: [Player]) -> some View {
        if !ps.isEmpty {
            SwiftUI.Section("\(title) (\(ps.count))") {
                ForEach(ps) { p in
                    NavigationLink(value: p.id) { PlayerRow(player: p) }
                }
            }
        }
    }
}

struct PlayerRow: View {
    let player: Player

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                Text(player.displayName).font(.headline)
                if let l = player.level {
                    Text("\(l)").font(.caption.monospacedDigit().bold())
                        .padding(.horizontal, 6).padding(.vertical, 1)
                        .background(player.isImmortal ? Color.purple.opacity(0.2) : Color.secondary.opacity(0.15), in: Capsule())
                }
                if let c = player.class, player.name != nil { Text(c).font(.caption).foregroundStyle(.secondary) }
                Spacer()
                badges
            }
            if let room = player.roomName, !room.isEmpty {
                Text("\(room) [\(String(player.room ?? -1))]").font(.subheadline).foregroundStyle(.secondary).lineLimit(1)
            } else if !player.isPlaying {
                Text("At the \(player.state) prompt").font(.subheadline).foregroundStyle(.secondary)
            }
            HStack {
                if !player.host.isEmpty { Text(player.host).font(.caption.monospaced()).foregroundStyle(.tertiary) }
                Spacer()
                if let idle = player.idleDescription { Text(idle).font(.caption).foregroundStyle(.orange) }
            }
        }
        .padding(.vertical, 2)
    }

    @ViewBuilder private var badges: some View {
        HStack(spacing: 4) {
            if player.fighting == true { Image(systemName: "flame.fill").foregroundStyle(.red) }
            if player.frozen == true { Image(systemName: "snowflake").foregroundStyle(.cyan) }
            if player.invisible == true { Image(systemName: "eye.slash").foregroundStyle(.secondary) }
            if player.snooping == true { Image(systemName: "binoculars.fill").foregroundStyle(.purple) }
            if player.noshout == true || player.notell == true || player.noemote == true {
                Image(systemName: "speaker.slash.fill").foregroundStyle(.orange)
            }
        }
        .font(.caption)
    }
}
