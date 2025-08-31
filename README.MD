# ScreenLock

A playful **screen lock utility** written in Rust. Ideal for quick self-locking or friendly warding-off of unintended usage. It locks your terminal session with a countdown timer and password protection.

---

## Features

- Displays a **countdown timer** at the top-left when active
- Password is set via the LOCK_PASSWORD env variable (defaults to 'password')
- Accepts an optional duration (e.g. `30s`, `5m`, `2h`) to define lock duration; defaults to 30s
- Prompts for a **password**, allowing repeated attempts while the timer runs
- Animated lock UI with emojis and GitHub credits
- Unlocks on either **timeout** or **correct password**

---

## Caveats

- Cannot use the character c in the password (C is a blocked key)
- Haven't tested special characters in the password
- All characters are auto converted to lowercase

---

## Usage

Run the compiled binary with optional `--for` argument:

```sh
screenlock --for 2m
```

- The lock UI appears instantly with an intro message.
- You can type and submit password attempts multiple times.
- Unlock when the timer runs out or you enter the correct password.

---

## Terminal Message on Startup

```
💻 This laptop has been intentionally left unattended.
🙅 Tampering with it is not appreciated.
🔒 Screen lock is active: Unlocks on timeout or correct password.
🌟 Send your love to this project: https://github.com/fa993/screenlock
```

---

## Installation

Assuming you have Rust installed:

```sh
git clone https://github.com/fa993/screenlock.git
cd screenlock
cargo build --release
```

Run the tool from `target/release/`:

```sh
./screenlock --for 1m
```

---

## Contributing

Contributions are welcome! Whether it's bug fixes, new features, improved UI, or custom timeout formats—feel free to open an issue or pull request.

---

## License

Specify your license here (e.g., MIT, Apache‑2.0, etc.).
