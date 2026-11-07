import Foundation

/// `GET /status`: public, no login. Also the "server" part of a snapshot.
struct ServerStatus: Codable, Equatable {
    var name: String
    var version: String
    var up: Bool
    var now: Int
    var boot: Int
    var uptime: Int
    var playing: Int
    var connections: Int
    var linkdead: Int
    var wizlock: Bool
    var world: WorldState
}

struct WorldState: Codable, Equatable {
    var time: GameTime
    var weather: Weather
}

struct GameTime: Codable, Equatable {
    var hour: Int
    var day: Int
    var month: Int
    var monthName: String
    var year: Int

    var description: String {
        let h = hour % 12 == 0 ? 12 : hour % 12
        let ampm = hour >= 12 ? "pm" : "am"
        return "\(h) \(ampm), day \(day) of the \(monthName), year \(year)"
    }
}

struct Weather: Codable, Equatable {
    var sky: String
    var sun: String
    var pressure: Int

    var symbol: String {
        switch sky {
        case "cloudless": return sun == "dark" ? "moon.stars.fill" : "sun.max.fill"
        case "cloudy": return sun == "dark" ? "cloud.moon.fill" : "cloud.sun.fill"
        case "raining": return "cloud.rain.fill"
        case "lightning": return "cloud.bolt.rain.fill"
        default: return "questionmark.circle"
        }
    }
}

/// The `{"t":"wiz","d":{...}}` reply an immortal gets for `\u{1}snap`.
struct Snapshot: Codable, Equatable {
    var server: ServerStatus
    var counts: Counts
    var players: [Player]
    var zones: [Zone]
    var viewerLevel: Int
    var log: [String]?
}

struct Counts: Codable, Equatable {
    var rooms: Int
    var mobs: Int
    var objects: Int
    var mobProtos: Int
    var objProtos: Int
    var playerFiles: Int
    var inCombat: Int
}

/// One connection, or one link-dead player still in the world.
struct Player: Codable, Equatable, Identifiable {
    var desc: Int
    var host: String
    var state: String
    var linkdead: Bool
    var name: String?
    var title: String?
    var level: Int?
    var `class`: String?
    var room: Int?
    var roomName: String?
    var zone: Int?
    var idle: Int?
    var hit: Int?
    var maxHit: Int?
    var mana: Int?
    var maxMana: Int?
    var move: Int?
    var maxMove: Int?
    var gold: Int?
    var exp: Int?
    var alignment: Int?
    var position: String?
    var fighting: Bool?
    var invisible: Bool?
    var frozen: Bool?
    var noshout: Bool?
    var notell: Bool?
    var noemote: Bool?
    var snooping: Bool?
    var switchedInto: String?

    var id: String { name.map { "p:\($0)" } ?? "d:\(desc)" }
    var displayName: String { name?.isEmpty == false ? name! : "(logging in)" }
    var isPlaying: Bool { state == "playing" }
    var isImmortal: Bool { (level ?? 0) >= 21 }

    /// Idle ticks are mud hours: 75 real seconds each.
    var idleDescription: String? {
        guard let idle, idle > 0 else { return nil }
        let minutes = idle * 75 / 60
        return minutes < 1 ? "idle" : "idle \(minutes)m"
    }
}

struct Zone: Codable, Equatable, Identifiable {
    var n: Int
    var name: String
    var bottom: Int
    var top: Int
    var age: Int
    var lifespan: Int
    var resetMode: Int
    var players: Int
    var mobs: Int

    var id: Int { n }

    var resetModeDescription: String {
        switch resetMode {
        case 0: return "Never resets"
        case 1: return "Resets when empty of players"
        default: return "Always resets"
        }
    }
}

enum Rank {
    static func name(for level: Int) -> String {
        switch level {
        case 24...: return "Implementor"
        case 23: return "God"
        case 22: return "Avatar"
        case 21: return "Immortal"
        default: return "Mortal"
        }
    }
}

extension JSONDecoder {
    static let diku: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()
}

func formatUptime(_ seconds: Int) -> String {
    let d = seconds / 86400, h = (seconds % 86400) / 3600, m = (seconds % 3600) / 60
    if d > 0 { return "\(d)d \(h)h" }
    if h > 0 { return "\(h)h \(m)m" }
    if m > 0 { return "\(m)m" }
    return "\(seconds)s"
}
