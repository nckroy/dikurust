import SwiftUI

/// Public health of the server from `/status`. Works without logging in.
struct StatusView: View {
    @Environment(StatusMonitor.self) private var monitor
    @Environment(AppSettings.self) private var settings
    @Environment(GameSession.self) private var session

    private let columns = [GridItem(.adaptive(minimum: 150), spacing: 12)]

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                hero
                if let s = monitor.status {
                    LazyVGrid(columns: columns, spacing: 12) {
                        StatTile(title: "Playing", value: "\(s.playing)", symbol: "person.fill")
                        StatTile(title: "Connections", value: "\(s.connections)", symbol: "network")
                        StatTile(title: "Link-dead", value: "\(s.linkdead)", symbol: "person.fill.questionmark", tint: .orange)
                        StatTile(title: "Uptime", value: formatUptime(s.uptime), symbol: "clock")
                    }
                    if s.wizlock {
                        Label("Wizlocked: the server is refusing all new connections.", systemImage: "lock.fill")
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding()
                            .background(.red.opacity(0.15), in: RoundedRectangle(cornerRadius: 12))
                    }
                    worldCard(s.world)
                    if let snap = session.snapshot {
                        LazyVGrid(columns: columns, spacing: 12) {
                            StatTile(title: "Mobiles", value: "\(snap.counts.mobs)", symbol: "pawprint")
                            StatTile(title: "Objects", value: "\(snap.counts.objects)", symbol: "shippingbox")
                            StatTile(title: "In combat", value: "\(snap.counts.inCombat)", symbol: "flame", tint: .red)
                            StatTile(title: "Player files", value: "\(snap.counts.playerFiles)", symbol: "person.text.rectangle")
                        }
                    }
                    footer(s)
                }
            }
            .padding()
        }
        .navigationTitle(settings.displayHost)
        .refreshable { await monitor.refresh(settings: settings) }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                if monitor.isChecking { ProgressView() }
                else {
                    Button { Task { await monitor.refresh(settings: settings) } } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
        }
    }

    private var hero: some View {
        let (color, title, detail): (Color, String, String) = {
            switch monitor.health {
            case .unknown: return (.gray, "Checking…", "Asking \(settings.displayHost) how it's doing.")
            case .up:
                let s = monitor.status
                return (s?.wizlock == true ? .orange : .green,
                        "The world is up",
                        s.map { "\($0.name) \($0.version), up since \(Date(timeIntervalSince1970: TimeInterval($0.boot)).formatted(date: .abbreviated, time: .shortened))" } ?? "")
            case .down(let why): return (.red, "Can't reach the world", why)
            }
        }()
        return HStack(alignment: .top, spacing: 14) {
            ZStack {
                Circle().fill(color.opacity(0.2)).frame(width: 52, height: 52)
                Circle().fill(color).frame(width: 20, height: 20)
            }
            VStack(alignment: .leading, spacing: 4) {
                Text(title).font(.title2.bold())
                Text(detail).font(.subheadline).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding()
        .background(.background.secondary, in: RoundedRectangle(cornerRadius: 16))
        .accessibilityElement(children: .combine)
    }

    private func worldCard(_ w: WorldState) -> some View {
        HStack(spacing: 14) {
            Image(systemName: w.weather.symbol)
                .symbolRenderingMode(.multicolor)
                .font(.system(size: 34))
                .frame(width: 52)
            VStack(alignment: .leading, spacing: 4) {
                Text("In the realm").font(.caption).foregroundStyle(.secondary)
                Text(w.time.description.capitalizedFirst).font(.headline)
                Text("Sky \(w.weather.sky), sun \(w.weather.sun), \(w.weather.pressure) mb")
                    .font(.subheadline).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding()
        .background(.background.secondary, in: RoundedRectangle(cornerRadius: 16))
    }

    private func footer(_ s: ServerStatus) -> some View {
        HStack {
            if let d = monitor.lastChecked {
                Text("Checked \(d.formatted(date: .omitted, time: .standard))")
            }
            if let l = monitor.latency {
                Text("· \(Int(l.components.attoseconds / 1_000_000_000_000_000 + l.components.seconds * 1000)) ms")
            }
            Spacer()
        }
        .font(.caption)
        .foregroundStyle(.secondary)
    }
}

extension String {
    var capitalizedFirst: String { prefix(1).uppercased() + dropFirst() }
}
