import Foundation
import Observation

/// Polls the public `/status` endpoint. Needs no login.
@MainActor
@Observable
final class StatusMonitor {
    enum Health: Equatable {
        case unknown
        case up
        case down(String)
    }

    private(set) var status: ServerStatus?
    private(set) var health: Health = .unknown
    private(set) var lastChecked: Date?
    private(set) var latency: Duration?
    private(set) var isChecking = false

    private var pollTask: Task<Void, Never>?

    func start(settings: AppSettings, every interval: Duration = .seconds(30)) {
        pollTask?.cancel()
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refresh(settings: settings)
                try? await Task.sleep(for: interval)
            }
        }
    }

    func stop() {
        pollTask?.cancel()
        pollTask = nil
    }

    func refresh(settings: AppSettings) async {
        guard let url = settings.statusURL else {
            health = .down("The server address isn't a valid URL.")
            return
        }
        isChecking = true
        defer { isChecking = false }
        var req = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 10)
        req.setValue("application/json", forHTTPHeaderField: "Accept")
        let clock = ContinuousClock()
        let start = clock.now
        do {
            let (data, resp) = try await URLSession.shared.data(for: req)
            latency = clock.now - start
            lastChecked = Date()
            guard let http = resp as? HTTPURLResponse else { throw URLError(.badServerResponse) }
            guard http.statusCode == 200 else {
                status = nil
                health = .down(http.statusCode == 404
                    ? "The server answered but has no /status endpoint. It may be running an older build."
                    : "The server answered HTTP \(http.statusCode).")
                return
            }
            status = try JSONDecoder.diku.decode(ServerStatus.self, from: data)
            health = .up
        } catch is CancellationError {
        } catch {
            lastChecked = Date()
            latency = nil
            status = nil
            health = .down(error.localizedDescription)
        }
    }
}
