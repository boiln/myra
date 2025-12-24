use std::time::Duration;
use windivert::layer::NetworkLayer;
use windivert::WinDivert;
use windivert_sys::WinDivertFlags;

fn main() {
    // Test various filter combinations
    let filters = [
        "true",
        "outbound",
        "outbound and !loopback",
        "!loopback",
        "loopback",
        "(outbound) and localPort != 1420 and remotePort != 1420",
    ];

    for filter in &filters {
        println!("\n=== Testing filter: {} ===", filter);

        let wd = match WinDivert::<NetworkLayer>::network(
            filter,
            0, // Priority 0
            WinDivertFlags::new(),
        ) {
            Ok(handle) => {
                println!("Handle opened successfully!");
                handle
            }
            Err(e) => {
                eprintln!("FAILED to open: {:?}", e);
                continue;
            }
        };

        let mut buffer = vec![0u8; 65535];
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(2);
        let mut count = 0;

        while start.elapsed() < timeout && count < 3 {
            match wd.recv(Some(&mut buffer)) {
                Ok(packet) => {
                    count += 1;
                    println!("Got packet: {} bytes", packet.data.len());
                }
                Err(e) => {
                    eprintln!("recv error: {:?}", e);
                    break;
                }
            }
        }

        if count == 0 {
            println!("NO packets received!");
        } else {
            println!("Received {} packets", count);
        }

        drop(wd); // Close handle before opening next
        std::thread::sleep(Duration::from_millis(100));
    }
}
