import socket
import time

def host_client_test():
    try:
        client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        client.settimeout(5)  
        
        print("[Host] Connecting...")
        client.connect(('localhost', 6666))
        
        test_data = b"Cross-VM TCP Test"
        print(f"[Host] Sending {len(test_data)} bytes")
        client.sendall(test_data)
        print("[Host] Send completed")
        

        client.shutdown(socket.SHUT_WR)
        
        received = bytearray()
        while len(received) < len(test_data):
            print("[Host] Waiting for response...")
            time.sleep(100)
            chunk = client.recv(1024)
            if not chunk:
                break
            received.extend(chunk)
            print(f"[Host] Received {len(chunk)} bytes (total {len(received)})")
        
        if bytes(received) == test_data:
            print("[Host] Test PASSED!")
        else:
            print(f"[Host] Test FAILED! Received {len(received)}/{len(test_data)} bytes")
        
    except Exception as e:
        print(f"[Host] Error: {str(e)}")
    finally:
        client.close()

if __name__ == "__main__":
    host_client_test()
    