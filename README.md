# vpidgeon

<img src="assets/pigeon.gif" alt="vpidgeon" width="140" align="right">

**Encrypted peer-to-peer terminal chat**

A terminal messaging app (TUI) written in Rust, where people chat with each other.

The lead needs a server (a small VM). It makes the ip exchange, and then it relays the connections: the lead connects through the VM to every member, because home routers often block a direct connection between two peers. The VM only passes the bytes along. Only the lead needs the VM. The others just need the app.

<br clear="right">

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

6. The server config is `vm_env.template`. The app fills in the code, the timeout and the maximum number of peers (100 seconds and 2 peers for a direct chat, 100 seconds and 20 peers for a group chat), so only change the port. The server stops waiting as soon as the chat is full:

   ```
   VPIDGEON_CODE=CHANGE_ME
   VPIDGEON_PORT=9000
   VPIDGEON_TIMEOUT=CHANGE_TIMEOUT
   VPIDGEON_MAX=CHANGE_MAX
   ```

## Run

You need [Rust](https://rustup.rs). On Windows it also asks for the Visual Studio Build Tools ("Desktop development with C++"). Then:

```bash
git clone https://github.com/stanislavstankovic05/vpidgeon.git
cd vpidgeon
cargo run
```

Use the arrow keys and Enter in the menu. Esc goes back, and Tab switches between fields.

- **Lead peer:** choose Direct chat or Group chat. The app starts the server on the VM and shows a code and a pre-shared key. Send the others the VM ip and the code. Give them the pre-shared key in person, never through the VM.
- **Joiner:** choose Join, write the VM ip, the code and the pre-shared key, and press Enter. Joiners don't need a VM or a `.env`.

Both of you have to join within 100 seconds of the lead opening the waiting screen. A direct chat starts as soon as the second person joins.

The messages are encrypted end to end with [snow](https://github.com/mcginty/snow) (Noise protocol). The VM only passes the encrypted bytes between the peers. The addresses are saved in `peers.txt`.

## TODO

- A settings page in the app for the VM (ip, name, ssh key, port), so the lead doesn't have to write the `.env` by hand.

## License

MIT, see `LICENSE`. The licenses of the crates it uses are in `THIRD_PARTY_LICENSES.md`.
