# Description 
Simple script to scrape Portainer for new containers, then update Homepage bookmarks and UptimeKuma with the new additions.
(I made this for a friend's server stack to automate the process. They have seperate single-board computers (Raspberry Pis) handling different processes, which is why `main.rs` includes code that SCP's a file over to the second machine.)

# Process
I used WSL2 Debian to code for a Pi4, using the cross-compiler native to Cargo to generate an executable.
Before you can cross-compile though, install the linker.
```bash
 rustup target add aarch64-unknown-linux-gnu 
```
and when building, run
```bash
cargo build --release --target=aarch64-unknown-linux-gnu
```
Then, the executable will (by default) be in `target/aarch64-unknown-linux-gnu/release/u_got_miau`

If you want to test this code, fill out `example_secret.json` with the relavent information and rename it `secret.json`, then go to `lib.rs` and
set "/path/to/secret.json" to the actual path where you save `secret.json`.

The second Pi is why there's a field called `pi_instance` in the secret file.
