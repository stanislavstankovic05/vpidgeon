# vpidgeon

A terminal messaging app (TUI) written in Rust, where people chat directly with each other, peer-to-peer.

**The lead needs a server (a small VM) to make the ip exchange.** Peers behind a router can't find each other's public address on their own, so the lead's VM gives the addresses to everyone who joins. After that, the peers talk directly and the VM is no longer used. Only the lead needs the VM. The others just need the app.

## Run

You need [Rust](https://rustup.rs). Then:

```bash
git clone https://github.com/stanislavstankovic05/vpidgeon.git
cd vpidgeon
cargo run
```

Use the arrow keys and Enter in the menu. Esc goes back, and Tab switches between fields.

**Start a chat (the lead):** choose Direct chat or Group chat. The app starts the server on your VM (see Settings) and shows a code and a pre-shared key. Send the others the VM ip and the code. Give them the pre-shared key in person, never through the VM.

**Join a chat:** choose Join, write the VM ip, the code and the pre-shared key, and press Enter. Joiners don't need a VM or a `.env`.

## Settings

Only the lead needs these settings. The VM is required: it is the server that does the ip exchange.

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

6. The server config is `vm_env.template`. The app fills in the code and the maximum number of peers (2 for a direct chat, 20 for a group), so only change the port and the timeout (in seconds):

   ```
   VPIDGEON_CODE=CHANGE_ME
   VPIDGEON_PORT=9000
   VPIDGEON_TIMEOUT=120
   VPIDGEON_MAX=CHANGE_MAX
   ```

The addresses of the peers are saved in `peers.txt`.
