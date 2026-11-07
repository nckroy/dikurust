import Foundation
import Observation
import Security

/// Where the server is and who to log in as. The password lives in the
/// Keychain; everything else in UserDefaults.
@Observable
final class AppSettings {
    static let defaultServer = "https://diku.earth"

    var server: String {
        didSet { UserDefaults.standard.set(server, forKey: "server") }
    }
    var character: String {
        didSet { UserDefaults.standard.set(character, forKey: "character") }
    }
    var autoConnect: Bool {
        didSet { UserDefaults.standard.set(autoConnect, forKey: "autoConnect") }
    }
    var password: String {
        didSet { Keychain.set(password, account: keychainAccount) }
    }

    init() {
        let d = UserDefaults.standard
        server = d.string(forKey: "server") ?? Self.defaultServer
        character = d.string(forKey: "character") ?? ""
        autoConnect = d.bool(forKey: "autoConnect")
        password = ""
        password = Keychain.get(account: keychainAccount) ?? ""
    }

    private var keychainAccount: String { "character-password" }

    var hasCredentials: Bool { !character.isEmpty && !password.isEmpty }

    /// https://host[:port] with any path stripped; http and bare hosts allowed.
    var baseURL: URL? {
        var s = server.trimmingCharacters(in: .whitespacesAndNewlines)
        if !s.contains("://") { s = "https://" + s }
        guard var c = URLComponents(string: s), c.host?.isEmpty == false else { return nil }
        c.path = ""
        c.query = nil
        c.fragment = nil
        return c.url
    }

    var statusURL: URL? { baseURL?.appendingPathComponent("status") }

    var webSocketURL: URL? {
        guard let base = baseURL, var c = URLComponents(url: base, resolvingAgainstBaseURL: false) else { return nil }
        c.scheme = c.scheme == "http" ? "ws" : "wss"
        c.path = "/ws"
        return c.url
    }

    var displayHost: String { baseURL?.host ?? server }
}

enum Keychain {
    private static let service = "earth.diku.wizard"

    static func get(account: String) -> String? {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var out: AnyObject?
        guard SecItemCopyMatching(q as CFDictionary, &out) == errSecSuccess, let data = out as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    static func set(_ value: String, account: String) {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        SecItemDelete(q as CFDictionary)
        guard !value.isEmpty else { return }
        var add = q
        add[kSecValueData as String] = Data(value.utf8)
        add[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        SecItemAdd(add as CFDictionary, nil)
    }
}
