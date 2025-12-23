# send incrementing packets containing numbers to given host
from __future__ import annotations

import argparse
import socket
from time import sleep

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="UDP/TCP packet sender")
    parser.add_argument("host", type=str)
    parser.add_argument("port", type=int)
    parser.add_argument(
        "-s", "--sleep", default=100, type=int, help="sleep time in ms", required=False
    )
    parser.add_argument(
        "--nosleep", help="use minimal sleep time instead of none", action="store_true"
    )
    parser.add_argument("--tcp", help="use tcp instead of udp", action="store_true")
    args = parser.parse_args()

    # Create socket based on protocol
    if args.tcp:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((args.host, args.port))
    else:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    print(f"Sending packets to {args.host}:{args.port}")

    cnt = 1
    try:
        while True:  # send till die
            # Create a small message
            message = f"{'-' * (1 + (cnt % 8))}\r\n"

            # Send the message
            if args.tcp:
                sock.sendall(message.encode())
            else:
                sock.sendto(message.encode(), (args.host, args.port))

            # Print counter
            print(cnt)
            cnt += 1

            if args.nosleep:
                # small delay to make it observable
                sleep(0.1)
            else:
                sleep(args.sleep / 1000.0)
    except KeyboardInterrupt:
        print("Stopping sender")
    finally:
        sock.close()
