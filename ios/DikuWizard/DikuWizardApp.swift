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
                    default:
                        break
                    }
                }
        }
    }
}
