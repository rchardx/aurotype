.PHONY: dev engine-dev setup

dev:
	~/.bun/bin/bun run tauri dev

engine-dev:
	cd engine && /home/rchardx/.local/bin/uv run python -m aurotype_engine

setup:
	~/.bun/bin/bun install
	cd engine && /home/rchardx/.local/bin/uv sync
