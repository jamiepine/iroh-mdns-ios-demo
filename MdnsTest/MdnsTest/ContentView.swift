import SwiftUI

struct ContentView: View {
    @StateObject private var peerManager = PeerManager.shared
    
    var body: some View {
        VStack(spacing: 30) {
            // Header
            Text("Bob (iOS)")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            Text("iroh mDNS Discovery Test")
                .font(.title2)
                .foregroundColor(.secondary)
            
            Divider()
                .padding(.vertical)
            
            // Status
            VStack(spacing: 15) {
                HStack {
                    Circle()
                        .fill(peerManager.isRunning ? Color.green : Color.gray)
                        .frame(width: 12, height: 12)
                    
                    Text(peerManager.isRunning ? "Listening for peers" : "Not started")
                        .font(.headline)
                }
                
                if peerManager.isRunning {
                    VStack(alignment: .leading, spacing: 8) {
                        Label("mDNS discovery active", systemImage: "wifi")
                        Label("Searching local network", systemImage: "network")
                        Label("Using swarm-discovery (PATCHED)", systemImage: "applelogo")
                    }
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .padding()
                    .background(Color.green.opacity(0.1))
                    .cornerRadius(8)
                }
            }
            
            Spacer()
            
            // Instructions
            VStack(alignment: .leading, spacing: 12) {
                Text("Instructions:")
                    .font(.headline)
                
                Group {
                    Text("1. Run desktop peer:")
                        .font(.caption)
                    Text("   cargo run --bin mdns-peer alice")
                        .font(.caption.monospaced())
                        .foregroundColor(.blue)
                    
                    Text("2. Watch Xcode console for:")
                        .font(.caption)
                    Text("   SUCCESS: Discovered peer 'alice'!")
                        .font(.caption.monospaced())
                        .foregroundColor(.green)
                    
                    Text("3. Check desktop console:")
                        .font(.caption)
                    Text("   SUCCESS: Discovered peer 'bob'!")
                        .font(.caption.monospaced())
                        .foregroundColor(.green)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding()
            .background(Color.gray.opacity(0.1))
            .cornerRadius(12)
            
            Spacer()
            
            // Control button
            Button(action: {
                if peerManager.isRunning {
                    peerManager.stop()
                } else {
                    _ = peerManager.start()
                }
            }) {
                Text(peerManager.isRunning ? "Stop" : "Start Discovery")
                    .font(.headline)
                    .foregroundColor(.white)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(peerManager.isRunning ? Color.red : Color.blue)
                    .cornerRadius(12)
            }
            .padding(.horizontal)
        }
        .padding()
        .onAppear {
            // Auto-start peer when app launches
            print("MdnsTest app launched")
            _ = peerManager.start()
        }
        .onDisappear {
            peerManager.stop()
        }
    }
}

#Preview {
    ContentView()
}