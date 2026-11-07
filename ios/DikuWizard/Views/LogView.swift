import SwiftUI

/// The last few hundred lines of the server's log. Implementors only:
/// it includes players' addresses.
struct LogView: View {
    @Environment(GameSession.self) private var session
    @State private var search = ""
    @State private var problemsOnly = false

    var body: some View {
        WizardGate(minimumLevel: 24) { snap in
            let lines = filtered(snap.log ?? [])
            List(lines.indices, id: \.self) { i in
                let line = lines[i]
                VStack(alignment: .leading, spacing: 2) {
                    if let (time, msg) = split(line) {
                        Text(time).font(.caption2.monospaced()).foregroundStyle(.secondary)
                        Text(msg).font(.system(.footnote, design: .monospaced)).foregroundStyle(tint(msg))
                    } else {
                        Text(line).font(.system(.footnote, design: .monospaced))
                    }
                }
                .textSelection(.enabled)
            }
            .listStyle(.plain)
            .searchable(text: $search, prompt: "Filter")
            .refreshable { session.requestSnapshot() }
            .overlay { if lines.isEmpty { ContentUnavailableView.search } }
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Toggle(isOn: $problemsOnly) { Image(systemName: "exclamationmark.triangle") }
                        .toggleStyle(.button)
                        .help("Only errors and warnings")
                }
                ToolbarItem(placement: .secondaryAction) {
                    ShareLink(item: (snap.log ?? []).joined(separator: "\n")) {
                        Label("Share log", systemImage: "square.and.arrow.up")
                    }
                }
            }
        }
        .navigationTitle("Server Log")
    }

    private func filtered(_ log: [String]) -> [String] {
        var l = Array(log.reversed())
        if problemsOnly { l = l.filter { tint(String($0)) != .primary } }
        let q = search.trimmingCharacters(in: .whitespaces)
        if !q.isEmpty { l = l.filter { $0.localizedCaseInsensitiveContains(q) } }
        return l
    }

    private func split(_ line: String) -> (String, String)? {
        guard let r = line.range(of: " :: ") else { return nil }
        return (String(line[..<r.lowerBound]), String(line[r.upperBound...]))
    }

    private func tint(_ msg: String) -> Color {
        if msg.contains("SYSERR") || msg.contains("panic") { return .red }
        if msg.contains("does not exist") || msg.contains("Non-Existing") || msg.contains("Undefd") { return .orange }
        if msg.contains("Shutdown") || msg.contains("wizlock") || msg.contains("advances") { return .purple }
        return .primary
    }
}
