# vpidgeon

A terminal messaging app (TUI) written in Rust, where people chat with each other.

The lead needs a server (a small VM). It makes the ip exchange, and then it relays the connections: the lead connects through the VM to every member, because home routers often block a direct connection between two peers. The VM only passes the bytes along. Only the lead needs the VM. The others just need the app.

## Settings

1. Create a VM on Google Cloud: `e2-micro`, region `us-central1`, `us-west1` or `us-east1`, Standard provisioning, Standard disk. Debian already has `python3`.

2. Open port 9000 in the VM firewall:

   ```bash
   gcloud compute firewall-rules create vpidgeon-9000 --direction=INGRESS --action=allow --rules=tcp:9000 --source-ranges=0.0.0.0/0
   ```

3. Create an SSH key:

   ```bash
   ssh-keygen -t ed25519 -f ~/.ssh/vpidgeon_key -N "" -C vpidgeon
   ```

4. In the console open the VM, click Edit, and add the output of `cat ~/.ssh/vpidgeon_key.pub` under SSH Keys. Save.

5. Create a `.env` file in this folder. Never commit it:

   ```
   VM_IP=<external ip of the VM>
   VM_NAME=vpidgeon
   SSH_key=/Users/<you>/.ssh/vpidgeon_key
   ```

   The external ip changes every time the VM is restarted, so update `VM_IP`.

6. The server config is `vm_env.template`. The app fills in the code, the timeout and the maximum number of peers (100 seconds and 2 peers for a direct chat, 300 seconds and 20 peers for a group chat), so only change the port. The server stops waiting as soon as the chat is full:

   ```
   VPIDGEON_CODE=CHANGE_ME
   VPIDGEON_PORT=9000
   VPIDGEON_TIMEOUT=CHANGE_TIMEOUT
   VPIDGEON_MAX=CHANGE_MAX
   ```

## Run

From this folder:

```bash
cargo run
```

- Lead peer: choose Direct chat or Group chat. The app starts the server on the VM and shows the code. Share the VM ip and the code with the others.
- Joiner: choose Join, write the VM ip and the code, press Enter.

The addresses are saved in `peers.txt`.
