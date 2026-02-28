.PHONY: dev engine-dev setup build-engine build

dev:
	~/.bun/bin/bun run tauri dev -f dev-sidecar

engine-dev:
	cd engine && /home/rchardx/.local/bin/uv run python -m aurotype_engine

setup:
	~/.bun/bin/bun install
	cd engine && /home/rchardx/.local/bin/uv sync

TRIPLE := $(shell rustc -vV | grep 'host:' | cut -d' ' -f2)

# Detect .exe suffix for Windows targets
ifeq ($(findstring windows,$(TRIPLE)),windows)
  EXE_EXT := .exe
else
  EXE_EXT :=
endif

build-engine:
	cd engine && /home/rchardx/.local/bin/uv run pyinstaller aurotype-engine.spec --noconfirm
	mkdir -p src-tauri/binaries
	cp engine/dist/aurotype-engine/aurotype-engine$(EXE_EXT) src-tauri/binaries/aurotype-engine-$(TRIPLE)$(EXE_EXT)

build: build-engine
	~/.bun/bin/bun run tauri build
