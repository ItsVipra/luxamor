# luxamor
A little webapp that lets your loved ones let you know they're thinking about you,
by changing the color of your lights for a few seconds.

## Usage
Since there are currently no binaries available you'll have to manually install and compile the app.
These instructions assume you have Rust, Cargo, and sqlite installed.

1. clone this repo `git clone https://github.com/ItsVipra/luxamor`
2. copy `luxamor.toml.example` to `luxamor.toml`
3. customize the example values
   - Obtain a HAAS token by going to http://homeassistant.local:8123/profile and scrolling all the way down
4. set a private key in `Rocket.toml`
   - the Rocket team advises to use `openssl rand -base64 32` to generate one
5. compile and run the app with `cargo run -r`
   - just `cargo run` is fine, if you want more console output
6. navigate to `localhost:5892/admin` to sign in to the admin interface

### Cross-compiling for ARM
If you want to run this app on ARM (e.g. raspberry pi) you can cross-compile it with [cross-rs](https://github.com/cross-rs/cross).
Usage is the same as cargo, so for ARM compilation execute `(sudo) cross build -r --target armv7-unknown-linux-gnueabihf
`.

---
# TODO
- Configureability
  - [X] Link length
  - [X] HAAS key
  - [X] HAAS URL
  - [X] Ping timeout
  - [X] Ping name
  - [X] Port
- Security
  - [X] Admin auth
  - [X] Rate Limit
- Code Quality
  - [ ] less String more str
  - [ ] less cloning
- Visuals
  - [ ] Nicer Ping template
  - [ ] Copy button on admin page
- Tests
  - [ ] All the things