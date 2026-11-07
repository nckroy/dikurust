import SwiftUI

struct SettingsView: View {
    @Environment(AppSettings.self) private var settings
    @Environment(GameSession.self) private var session
    @Environment(StatusMonitor.self) private var monitor

    var body: some View {
        @Bindable var settings = settings
        Form {
            SwiftUI.Section {
                TextField("Server", text: $settings.server)
                    .keyboardType(.URL)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .onSubmit { Task { await monitor.refresh(settings: settings) } }
                Button("Use diku.earth") { settings.server = AppSettings.defaultServer }
                    .disabled(settings.server == AppSettings.defaultServer)
            } header: {
                Text("Server")
            } footer: {
                Text("Status comes from \(settings.statusURL?.absoluteString ?? "–"); the game from \(settings.webSocketURL?.absoluteString ?? "–").")
            }

            SwiftUI.Section {
                TextField("Character name", text: $settings.character)
                    .textContentType(.username)
                    .textInputAutocapitalization(.words)
                    .autocorrectionDisabled()
                SecureField("Password", text: $settings.password)
                    .textContentType(.password)
                Toggle("Connect when the app opens", isOn: $settings.autoConnect)
            } header: {
                Text("Your immortal")
            } footer: {
                Text("The password is kept in this device's Keychain and only ever sent to the game's login prompt. You can't be logged in from the web client and the app at the same time.")
            }

            SwiftUI.Section {
                switch session.phase {
                case .playing, .connecting, .loggingIn:
                    LabeledContent("Status", value: session.phase == .playing ? "Logged in as \(session.characterName)" : "Logging in…")
                    Button("Quit the game (saves)") { session.quit() }
                    Button("Drop link", role: .destructive) { session.disconnect() }
                case .failed(let why):
                    Text(why).foregroundStyle(.red)
                    Button("Connect") { session.connect(settings: settings) }.disabled(!settings.hasCredentials)
                case .disconnected:
                    Button("Connect") { session.connect(settings: settings) }.disabled(!settings.hasCredentials)
                }
            }

            SwiftUI.Section("About") {
                Text("DikuMUD was created by Sebastian Hammer, Michael Seifert, Hans Henrik Stærfeldt, Tom Madsen and Katja Nyboe. Its license applies to this port and to this app: no profit of any kind, and keep the credits.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
        }
        .navigationTitle("Settings")
    }
}
