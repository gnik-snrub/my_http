# 🛠️ Raw Rust: A Framework-Free HTTP Server

This is a fully custom-built HTTP server written in Rust — designed from the ground up to deepen backend systems knowledge and serve as a portfolio-grade capstone project. It handles raw socket connections, supports TLS via `rustls`, includes a modular middleware layer, serves static files, and features its own thread pool implementation — all built with **minimal external dependencies**.

---

## 🚀 Features

- 🔐 **TLS support** with self-signed certificates or PEM files
- 🧱 **Custom middleware system**
- 🍪 **Cookie serialization** and support for sessions
- 📂 **Static file serving** with automatic MIME type resolution
- 🔄 **Router layer** (plug-and-play design)
- 🧵 **Custom thread pool implementation** (manual job queue; uses Tokio only for runtime handle — not async logic)
- 🔁 **Keep-alive** connection support
- 🧪 **Comprehensive integration and unit test suite**
- 📦 **Minimal dependencies** — focus is on learning and control

> ❗ Currently supports HTTP/1.1 only — no HTTP/2 (yet!)

---

## 🔧 Running the Server

```bash
cargo run
```

By default, the server looks for `cert.pem` and `key.pem` in the project root. The bind address is configurable via a .env file and can be set to any valid host:port combination.

You can configure this using a `.env` file:

```
TLS_CERT_PATH=./path/to/cert.pem
TLS_KEY_PATH=./path/to/key.pem
BIND_ADDRESS=0.0.0.0:PORT  # Set to any valid host:port
```

Once running, the server logs its bound address and begins listening for incoming HTTPS connections.

Example output:
```
Server listening on https://127.0.0.1:7878
```

---

## 🧪 Running Tests

```bash
cargo test
```

Test coverage includes:
- HTTP lifecycle handling
- Middleware dispatch behavior
- Cookie formatting and expiry
- Static file resolution and MIME logic
- Thread pool job execution
- TLS server/client interaction

---

## 📚 Goals and Design Philosophy

This project was built to:

- Replace abstraction with understanding
- Showcase backend fluency in pure Rust
- Explore low-level networking, TLS, and threading patterns
- Provide a modular foundation for future projects

No frameworks. No shortcuts. Every line of logic was deliberately implemented and verified.

---

## ✍️ Author

Built by Josiah Morris as a portfolio project to showcase real-world systems programming ability in Rust.  
Open to feedback, contributions, and conversation.

---

## 📄 License

MIT — do whatever you want, just don't claim you wrote it.
