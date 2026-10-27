# vpidgeon

A terminal messaging app (TUI) written in Rust, where people chat directly with each other, peer-to-peer.

## Run

You need [Rust](https://rustup.rs). Then:

```bash
git clone https://github.com/stanislavstankovic05/vpidgeon.git
cd vpidgeon
cargo run
```

Use the arrow keys and Enter in the menu. Esc goes back, and Tab switches between fields.

**Start a chat:** choose Direct chat (2 people) or Group chat.
- Without a VM: the app shows an address. Send it to the others.
- With a VM (see Settings): the app starts the server on the VM and shows a code. Send the others the VM ip and the code.
- Either way, the app also shows a pre-shared key. Give it to the others in person, never through the VM.

**Join a chat:** choose Join.
- Without a VM: write the lead's `ip:port` and the pre-shared key. The password is not needed.
- With a VM: write the VM ip, the code and the pre-shared key.
- Press Enter to join.

Without a VM it works on the same network. Over the internet, the lead has to forward port 9001 on the router, or use a VM.

## Settings

The VM is optional. Without the `.env` settings below, the lead runs in direct mode: the app listens on port 9001 and shows the address (`ip:9001`). Skip steps 1 to 6 if you don't use a VM.

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

With a VM, the addresses are saved in `peers.txt`.
