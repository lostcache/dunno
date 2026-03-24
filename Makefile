.PHONY: ui-install ui-build build dev clean

# Install JS dependencies (run once or after package.json changes)
ui-install:
	cd ui && npm install

# Build the Svelte UI into static/dist/
ui-build:
	cd ui && npm run build

# Full production build: UI first, then Rust binaries
build: ui-build
	cargo build --release --bin dn
	cargo build --release --bin dn-server

# Development workflow:
#   surreal start --bind 127.0.0.1:8000 --username root --password root surrealkv://~/.local/share/dunno/data.db
#   Set backend = "local-server" in dunno.toml (points at a running SurrealDB instance)
#   terminal 1: cargo run --bin dn-server -- --no-open
#   terminal 2: make dev  (Vite on :5173, proxies /api to :7700)
dev:
	cd ui && npm run dev

clean:
	rm -rf static/dist
	cd ui && rm -rf node_modules/.cache
