import SwiftUI

/// The game itself: a scrollback and a command line.
struct ConsoleView: View {
    @Environment(GameSession.self) private var session
    @Environment(AppSettings.self) private var settings
    @State private var input = ""
    @State private var history: [String] = []
    @FocusState private var focused: Bool

    private let quick = ["look", "who", "users", "where", "score", "stat room", "wizhelp", "save"]
    private static let phosphor = Color(red: 0.55, green: 1.0, blue: 0.6)

    var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(session.lines) { line in
                            Text(line.text)
                                .font(.system(size: 13, design: .monospaced))
                                .foregroundStyle(color(line.kind))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .textSelection(.enabled)
                                .id(line.id)
                        }
                    }
                    .padding(10)
                }
                .background(Color.black)
                .defaultScrollAnchor(.bottom)
                .onChange(of: session.lines.last?.id) { _, id in
                    if let id { withAnimation(.easeOut(duration: 0.15)) { proxy.scrollTo(id, anchor: .bottom) } }
                }
                .onTapGesture { focused = true }
            }

            if session.isPlaying {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack {
                        ForEach(quick, id: \.self) { cmd in
                            Button(cmd) { submit(cmd) }
                                .buttonStyle(.bordered).controlSize(.small)
                        }
                    }
                    .padding(.horizontal, 10).padding(.vertical, 6)
                }
                .background(.bar)
            }

            HStack(spacing: 8) {
                Menu {
                    ForEach(history.suffix(20).reversed(), id: \.self) { h in
                        Button(h) { input = h }
                    }
                } label: {
                    Image(systemName: "clock.arrow.circlepath")
                }
                .disabled(history.isEmpty)

                Group {
                    if session.echo {
                        TextField(session.isPlaying ? "Command" : "Not connected", text: $input)
                    } else {
                        SecureField("Hidden input", text: $input)
                    }
                }
                .textFieldStyle(.roundedBorder)
                .font(.system(.body, design: .monospaced))
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .submitLabel(.send)
                .focused($focused)
                .onSubmit { submit(input) }
                .disabled(!session.phase.isActive)

                Button { submit(input) } label: { Image(systemName: "return") }
                    .buttonStyle(.borderedProminent)
                    .disabled(!session.phase.isActive)
            }
            .padding(10)
            .background(.bar)
        }
        .navigationTitle("Console")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                Menu {
                    if session.phase.isActive {
                        Button("Quit the game (saves)", systemImage: "rectangle.portrait.and.arrow.right") { session.quit() }
                        Button("Drop link", systemImage: "bolt.horizontal") { session.disconnect() }
                    } else {
                        Button("Connect", systemImage: "bolt.horizontal.fill") { session.connect(settings: settings) }
                            .disabled(!settings.hasCredentials)
                    }
                    Divider()
                    Button("Clear scrollback", systemImage: "trash") { session.clearTranscript() }
                } label: {
                    Image(systemName: session.phase.isActive ? "bolt.horizontal.fill" : "bolt.horizontal")
                }
            }
        }
    }

    private func submit(_ text: String) {
        guard session.phase.isActive else { return }
        session.send(text)
        if session.echo, !text.isEmpty, history.last != text { history.append(text) }
        input = ""
    }

    private func color(_ kind: GameSession.Line.Kind) -> Color {
        switch kind {
        case .output: return Self.phosphor
        case .input: return .white
        case .system: return .yellow
        }
    }
}
