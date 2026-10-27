import os
import socket
import threading
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


def close_socket(sock):
    try:
        sock.shutdown(socket.SHUT_RDWR)
    except OSError:
        pass
    sock.close()


def copy(source, target):
    while True:
        try:
            data = source.recv(4096)
        except OSError:
            data = b""
        if not data:
            break
        try:
            target.sendall(data)
        except OSError:
            break
    close_socket(source)
    close_socket(target)


def handle(client):
    global paired
    client.settimeout(5)
    try:
        hello = client.recv(256)
    except OSError:
        hello = b""
    parts = hello.decode(errors="ignore").split()

    valid = False
    if len(parts) == 4 and parts[0] == "RELAY" and parts[1].encode() == code:
        if parts[2] == "LEAD" or parts[2] == "MEMBER":
            if parts[3].isdigit() and 1 <= int(parts[3]) <= expected:
                valid = True
    if not valid:
        close_socket(client)
        return

    role = parts[2]
    index = int(parts[3])
    other = None
    with lock:
        entry = waiting.get(index)
        if entry is None:
            waiting[index] = (role, client)
        elif entry[0] == role:
            close_socket(entry[1])
            waiting[index] = (role, client)
        else:
            other = entry[1]
            del waiting[index]
            paired = paired + 1
    if other is None:
        return

    try:
        client.sendall(b"PAIRED\n")
        other.sendall(b"PAIRED\n")
    except OSError:
        close_socket(client)
        close_socket(other)
        return
    client.settimeout(None)
    other.settimeout(None)
    client.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    other.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    first = threading.Thread(target=copy, args=(client, other))
    second = threading.Thread(target=copy, args=(other, client))
    pipes.append(first)
    pipes.append(second)
    first.start()
    second.start()


expected = len(clients) - 1
paired = 0
waiting = {}
pipes = []
handlers = []
lock = threading.Lock()
relay_deadline = time.time() + 60

while paired < expected and time.time() < relay_deadline:
    time_left = relay_deadline - time.time()
    if time_left < 0.1:
        time_left = 0.1
    listener.settimeout(time_left)

    try:
        client, address = listener.accept()
    except socket.timeout:
        break

    handler = threading.Thread(target=handle, args=(client,))
    handlers.append(handler)
    handler.start()

listener.close()

for handler in handlers:
    handler.join()

for entry in waiting.values():
    close_socket(entry[1])

for pipe in pipes:
    pipe.join()
