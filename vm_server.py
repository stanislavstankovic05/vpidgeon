import os
import socket
import time

folder = os.path.dirname(os.path.abspath(__file__))
env_path = os.path.join(folder, ".env")
env_file = open(env_path)
env = {}
for line in env_file:
    parts = line.strip().split("=", 1)
    env[parts[0]] = parts[1]
env_file.close()

code = env["VPIDGEON_CODE"].encode()
listen_port = int(env["VPIDGEON_PORT"])
timeout = int(env["VPIDGEON_TIMEOUT"])
max_members = int(env["VPIDGEON_MAX"])
deadline = time.time() + timeout

listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("0.0.0.0", listen_port))
listener.listen(16)

clients = []
addresses = []

while time.time() < deadline and len(clients) < max_members:
    time_left = deadline - time.time()
    if time_left < 0.1:
        time_left = 0.1
    listener.settimeout(time_left)

    try:
        client, address = listener.accept()
    except socket.timeout:
        break

    client.settimeout(5)
    try:
        data = client.recv(256)
    except OSError:
        data = b""
    data = data.strip()

    if data == code:
        clients.append(client)
        addresses.append(address)
    else:
        client.close()

listener.close()

body = ""
position = 0
for address in addresses:
    text = address[0] + ":" + str(address[1])
    if position == 0:
        body = body + "LEAD " + text + "\n"
    else:
        body = body + "MEMBER " + text + "\n"
    position = position + 1

position = 0
for client in clients:
    address = addresses[position]
    own = "YOU " + address[0] + ":" + str(address[1]) + "\n"
    message = own + body
    try:
        client.sendall(message.encode())
    except OSError:
        pass
    client.close()
    position = position + 1
