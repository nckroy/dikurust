import SwiftUI

enum Section: String, CaseIterable, Identifiable, Hashable {
    case status, players, world, catalog, console, log, server, settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .status: return "Status"
        case .players: return "Players"
        case .world: return "World"
        case .catalog: return "Catalog"
        case .console: return "Console"
        case .log: return "Server Log"
        case .server: return "Server"
        case .settings: return "Settings"
        }
    }

    var symbol: String {
        switch self {
        case .status: return "waveform.path.ecg"
        case .players: return "person.3"
        case .world: return "map"
        case .catalog: return "books.vertical"
        case .console: return "terminal"
        case .log: return "doc.text.magnifyingglass"
        case .server: return "bolt.shield"
        case .settings: return "gearshape"
        }
    }

    @ViewBuilder var view: some View {
        switch self {
        case .status: StatusView()
        case .players: PlayersView()
        case .world: WorldView()
        case .catalog: CatalogView()
        case .console: ConsoleView()
        case .log: LogView()
        case .server: ServerControlView()
        case .settings: SettingsView()
        }
    }
}

/// Tabs on iPhone; a sidebar on iPad and other wide windows.
struct RootView: View {
    @Environment(\.horizontalSizeClass) private var sizeClass
    @State private var selection: Section? = .status
    @State private var tab: Section = .status
    @State private var columns: NavigationSplitViewVisibility = .all

    var body: some View {
        if sizeClass == .regular {
            NavigationSplitView(columnVisibility: $columns) {
                List(Section.allCases, selection: $selection) { s in
                    Label(s.title, systemImage: s.symbol).tag(s)
                }
                .navigationTitle("Diku Wizard")
                .safeAreaInset(edge: .bottom) { ConnectionBadge().padding() }
            } detail: {
                NavigationStack {
                    (selection ?? .status).view
                }
            }
            .navigationSplitViewStyle(.balanced)
        } else {
            // Five tabs, the last one a menu of the rest: the system "More"
            // tab nests navigation stacks and loses its place.
            TabView(selection: $tab) {
                ForEach(Section.phoneTabs) { s in
                    NavigationStack { s.view }
                        .tabItem { Label(s.title, systemImage: s.symbol) }
                        .tag(s)
                }
                NavigationStack { WizardMenu() }
                    .tabItem { Label("Wizard", systemImage: "wand.and.stars") }
                    .tag(Section.settings)
            }
        }
    }
}

extension Section {
    static let phoneTabs: [Section] = [.status, .players, .world, .console]
    static let phoneMenu: [Section] = [.catalog, .server, .log, .settings]
}

/// The phone's fifth tab: connection, then the less-used sections.
struct WizardMenu: View {
    var body: some View {
        List {
            SwiftUI.Section { ConnectionBadge().listRowInsets(EdgeInsets()).listRowBackground(Color.clear) }
            SwiftUI.Section {
                ForEach(Section.phoneMenu) { s in
                    NavigationLink { s.view } label: { Label(s.title, systemImage: s.symbol) }
                }
            }
        }
        .navigationTitle("Wizard")
    }
}

/// Small "who am I connected as" pill.
struct ConnectionBadge: View {
    @Environment(GameSession.self) private var session

    var body: some View {
        HStack(spacing: 8) {
            Circle().fill(color).frame(width: 9, height: 9)
            Text(label).font(.footnote).lineLimit(1)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 12).padding(.vertical, 8)
        .background(.thinMaterial, in: Capsule())
    }

    private var color: Color {
        switch session.phase {
        case .playing: return session.isImmortal ? .green : .yellow
        case .connecting, .loggingIn: return .orange
        case .failed: return .red
        case .disconnected: return .gray
        }
    }

    private var label: String {
        switch session.phase {
        case .playing:
            if let s = session.snapshot { return "\(session.characterName) · \(Rank.name(for: s.viewerLevel))" }
            return session.characterName
        case .connecting: return "Connecting…"
        case .loggingIn: return "Logging in…"
        case .failed: return "Login failed"
        case .disconnected: return "Not connected"
        }
    }
}
