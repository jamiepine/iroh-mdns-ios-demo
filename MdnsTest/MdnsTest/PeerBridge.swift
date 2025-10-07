import Foundation
import Combine

// C functions from mdns-peer framework
@_silgen_name("bob_start")
func bob_start() -> Bool

@_silgen_name("bob_stop")
func bob_stop()

/// Manager for mDNS discovery peer
class PeerManager: ObservableObject {
    static let shared = PeerManager()
    
    @Published var isRunning = false
    
    private init() {}
    
    func start() -> Bool {
        guard !isRunning else {
            print("Warning: Peer is already running")
            return true
        }
        
        print("Starting peer...")
        let success = bob_start()
        
        if success {
            isRunning = true
            print("Peer started successfully")
            print("Watch Xcode console for Rust tracing logs")
        } else {
            print("Error: Peer failed to start")
        }
        
        return success
    }
    
    func stop() {
        guard isRunning else {
            print("Warning: Peer is not running")
            return
        }
        
        print("Stopping peer...")
        bob_stop()
        isRunning = false
        print("Peer stopped")
    }
    
    deinit {
        if isRunning {
            bob_stop()
        }
    }
}
