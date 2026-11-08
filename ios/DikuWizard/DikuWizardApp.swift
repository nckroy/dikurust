import SwiftUI

@main
struct DikuWizardApp: App {
    @State private var settings = AppSettings()
    @State private var monitor = StatusMonitor()
    @State private var session = GameSession()
    @Environment(\.scenePhase) private var scenePhase

    var body: some Scene {
        WindowGroup {
            RootView()
                .environment(settings)
                .environment(monitor)
                .environment(session)
                .onAppear {
                    monitor.start(settings: settings)
                    if settings.autoConnect && settings.hasCredentials {
                        session.connect(settings: settings)
                    }
                }
                .onChange(of: scenePhase) { _, phase in
                    switch phase {
                    case .active:
                        monitor.start(settings: settings)
                        if session.phase == .disconnected, settings.autoConnect, settings.hasCredentials {
                            session.connect(settings: settings)
                        }
                    case .background:
                        monitor.stop()
                        quitOnClose()
                    default:
                        break
                    }
                }
        }
    }

    /// Leaving the app quits the game, so the character doesn't stay in the
    /// world link-dead. iOS gives a backgrounded app a short grace period;
    /// the background task keeps it running until `quit` has gone out and
    /// the server has had a moment to save.
    private func quitOnClose() {
        guard session.phase.isActive else { return }
        let app = UIApplication.shared
        let id = app.beginBackgroundTask(withName: "Quit the game")
        Task {
            await session.leave()
            if id != .invalid { app.endBackgroundTask(id) }
        }
    }
}
