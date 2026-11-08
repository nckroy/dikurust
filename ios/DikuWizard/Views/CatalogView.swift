import SwiftUI

/// Every mobile, object and room prototype, searchable by name or number,
/// so you can load or visit something without knowing its number.
struct CatalogView: View {
    var body: some View {
        WizardGate { snap in
            CatalogList(snapshot: snap)
        }
        .navigationTitle("Catalog")
    }
}

enum CatalogKind: String, CaseIterable, Identifiable {
    case mobs = "Mobiles", objects = "Objects", rooms = "Rooms"
    var id: String { rawValue }
}

private struct CatalogList: View {
    @Environment(GameSession.self) private var session
    let snapshot: Snapshot
    @State private var kind: CatalogKind = .mobs
    @State private var search = ""
    @State private var zone: Int?

    var body: some View {
        Group {
            if let catalog = session.catalog {
                list(catalog)
            } else if let err = session.catalogError {
                ContentUnavailableView {
                    Label("No catalog", systemImage: "books.vertical")
                } description: {
                    Text(err)
                } actions: {
                    Button("Try again") { session.requestCatalog(force: true) }
                        .buttonStyle(.borderedProminent)
                }
            } else {
                ProgressView("Fetching every prototype…")
            }
        }
        .task { session.requestCatalog() }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Picker("Zone", selection: $zone) {
                        Text("All zones").tag(Int?.none)
                        ForEach(snapshot.zones) { z in
                            Text(z.name).tag(Int?.some(z.n))
                        }
                    }
                } label: {
                    Image(systemName: zone == nil
                          ? "line.3.horizontal.decrease.circle"
                          : "line.3.horizontal.decrease.circle.fill")
                }
                .accessibilityLabel("Filter by zone")
            }
        }
    }

    private func list(_ c: Catalog) -> some View {
        List {
            SwiftUI.Section {
                Picker("Kind", selection: $kind) {
                    ForEach(CatalogKind.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
                .listRowBackground(Color.clear)
                .listRowInsets(EdgeInsets())
            }
            SwiftUI.Section(header) {
                switch kind {
                case .mobs:
                    let rows = c.mobs.filter { inZone($0.zone) && matches($0.vnum, $0.keywords, $0.short) }
                    ForEach(rows) { m in
                        NavigationLink { MobDetail(mob: m, level: snapshot.viewerLevel, zoneName: zoneName(m.zone)) } label: {
                            CatalogRow(vnum: m.vnum, title: m.short,
                                       detail: "level \(m.level) · \(m.keywords)", count: m.count)
                        }
                    }
                    empty(rows.isEmpty)
                case .objects:
                    let rows = c.objects.filter { inZone($0.zone) && matches($0.vnum, $0.keywords, $0.short) }
                    ForEach(rows) { o in
                        NavigationLink { ObjDetail(obj: o, level: snapshot.viewerLevel, zoneName: zoneName(o.zone)) } label: {
                            CatalogRow(vnum: o.vnum, title: o.short,
                                       detail: ([o.type] + (o.headline.map { [$0] } ?? [])).joined(separator: " · "),
                                       count: o.count)
                        }
                    }
                    empty(rows.isEmpty)
                case .rooms:
                    let rows = c.rooms.filter { inZone($0.zone) && matches($0.vnum, $0.name, "") }
                    ForEach(rows) { r in
                        NavigationLink { RoomDetail(room: r, zoneName: zoneName(r.zone)) } label: {
                            CatalogRow(vnum: r.vnum, title: r.name, detail: r.sector, count: nil)
                        }
                    }
                    empty(rows.isEmpty)
                }
            }
        }
        .searchable(text: $search, prompt: "Name, keyword or number")
        .refreshable { session.requestCatalog(force: true) }
    }

    private var header: String {
        guard let z = zone else { return "All zones" }
        return zoneName(z) ?? "Zone \(z)"
    }

    @ViewBuilder private func empty(_ isEmpty: Bool) -> some View {
        if isEmpty {
            Text(search.isEmpty ? "Nothing in this zone." : "No matches.")
                .foregroundStyle(.secondary)
        }
    }

    private func inZone(_ z: Int) -> Bool { zone == nil || zone == z }

    /// A number matches by prefix; words match anywhere in the keywords or name.
    private func matches(_ vnum: Int, _ a: String, _ b: String) -> Bool {
        let q = search.trimmingCharacters(in: .whitespaces)
        if q.isEmpty { return true }
        if q.allSatisfy(\.isNumber) { return String(vnum).hasPrefix(q) }
        return a.localizedCaseInsensitiveContains(q) || b.localizedCaseInsensitiveContains(q)
    }

    private func zoneName(_ z: Int) -> String? {
        snapshot.zones.first { $0.n == z }?.name
    }
}

private struct CatalogRow: View {
    let vnum: Int
    let title: String
    let detail: String
    let count: Int?

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 10) {
            Text(verbatim: "\(vnum)")
                .font(.callout.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(minWidth: 44, alignment: .trailing)
            VStack(alignment: .leading, spacing: 2) {
                Text(title).lineLimit(1)
                Text(detail).font(.caption).foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer(minLength: 0)
            if let count, count > 0 {
                Text("\(count)")
                    .font(.caption.monospacedDigit())
                    .padding(.horizontal, 6).padding(.vertical, 2)
                    .background(.background.secondary, in: Capsule())
                    .accessibilityLabel("\(count) in the world")
            }
        }
    }
}

// MARK: details

private struct MobDetail: View {
    @Environment(GameSession.self) private var session
    @State private var runner = CommandRunner()
    let mob: MobProto
    let level: Int
    let zoneName: String?

    var body: some View {
        List {
            SwiftUI.Section {
                Text(mob.long).font(.callout)
            }
            SwiftUI.Section {
                if level >= 22 {
                    run("Load one here", "pawprint", "load char \(mob.vnum)")
                }
                run("Where are they?", "location.magnifyingglass", "where \(mob.keyword)")
            }
            SwiftUI.Section("Prototype") {
                LabeledContent("Number", value: String(mob.vnum))
                LabeledContent("Keywords", value: mob.keywords)
                LabeledContent("Level", value: "\(mob.level)")
                LabeledContent("Hit points", value: mob.hp)
                LabeledContent("Alignment", value: "\(mob.alignment)")
                LabeledContent("Gold", value: "\(mob.gold)")
                LabeledContent("Experience", value: "\(mob.exp)")
                if !mob.flags.isEmpty { LabeledContent("Flags", value: mob.flags.joined(separator: ", ")) }
                if !mob.spec.isEmpty { LabeledContent("Special", value: mob.spec) }
                LabeledContent("In the world now", value: "\(mob.count)")
                if let zoneName { LabeledContent("Zone", value: zoneName) }
            }
        }
        .navigationTitle(mob.short)
        .navigationBarTitleDisplayMode(.inline)
        .sheet(item: $runner.result) { CommandResultSheet(result: $0) }
    }

    private func run(_ title: String, _ symbol: String, _ command: String) -> some View {
        Button { runner.run(command, on: session) } label: { Label(title, systemImage: symbol) }
            .disabled(runner.running != nil)
    }
}

private struct ObjDetail: View {
    @Environment(GameSession.self) private var session
    @State private var runner = CommandRunner()
    let obj: ObjProto
    let level: Int
    let zoneName: String?

    var body: some View {
        List {
            if !obj.long.isEmpty {
                SwiftUI.Section { Text(obj.long).font(.callout) }
            }
            SwiftUI.Section {
                if level >= 22 {
                    run("Load one here", "shippingbox", "load obj \(obj.vnum)")
                }
                run("Where are they?", "location.magnifyingglass", "where \(obj.keyword)")
            }
            SwiftUI.Section("Prototype") {
                LabeledContent("Number", value: String(obj.vnum))
                LabeledContent("Keywords", value: obj.keywords)
                LabeledContent("Type", value: obj.type)
                if let h = obj.headline { LabeledContent("Stats", value: h) }
                if !obj.wornOn.isEmpty { LabeledContent("Worn on", value: obj.wornOn.joined(separator: ", ")) }
                LabeledContent("Can be picked up", value: obj.wear.contains("take") ? "yes" : "no")
                if !obj.extra.isEmpty { LabeledContent("Flags", value: obj.extra.joined(separator: ", ")) }
                LabeledContent("Values", value: obj.values.map(String.init).joined(separator: "  "))
                LabeledContent("Weight", value: "\(obj.weight)")
                LabeledContent("Cost", value: "\(obj.cost)")
                LabeledContent("Rent per day", value: "\(obj.rent)")
                LabeledContent("In the world now", value: "\(obj.count)")
                if let zoneName { LabeledContent("Zone", value: zoneName) }
            }
        }
        .navigationTitle(obj.short)
        .navigationBarTitleDisplayMode(.inline)
        .sheet(item: $runner.result) { CommandResultSheet(result: $0) }
    }

    private func run(_ title: String, _ symbol: String, _ command: String) -> some View {
        Button { runner.run(command, on: session) } label: { Label(title, systemImage: symbol) }
            .disabled(runner.running != nil)
    }
}

private struct RoomDetail: View {
    @Environment(GameSession.self) private var session
    @State private var runner = CommandRunner()
    let room: RoomProto
    let zoneName: String?

    var body: some View {
        List {
            SwiftUI.Section {
                run("Go there", "figure.walk.arrival", "goto \(room.vnum)")
                run("Look there", "binoculars", "at \(room.vnum) look")
                run("Stat the room", "info.circle", "at \(room.vnum) stat room")
            }
            SwiftUI.Section("Room") {
                LabeledContent("Number", value: String(room.vnum))
                LabeledContent("Sector", value: room.sector)
                if !room.flags.isEmpty { LabeledContent("Flags", value: room.flags.joined(separator: ", ")) }
                if let zoneName { LabeledContent("Zone", value: zoneName) }
            }
        }
        .navigationTitle(room.name)
        .navigationBarTitleDisplayMode(.inline)
        .sheet(item: $runner.result) { CommandResultSheet(result: $0) }
    }

    private func run(_ title: String, _ symbol: String, _ command: String) -> some View {
        Button { runner.run(command, on: session) } label: { Label(title, systemImage: symbol) }
            .disabled(runner.running != nil)
    }
}
