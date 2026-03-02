.PHONY: dev engine-dev setup build-engine build

dev:
	bun run tauri dev -f dev-sidecar

engine-dev:
	cd engine && uv run python -m aurotype_engine

setup:
	bun install
	cd engine && uv sync

TRIPLE := $(shell rustc -vV | grep 'host:' | cut -d' ' -f2)

# Detect .exe suffix for Windows targets
ifeq ($(findstring windows,$(TRIPLE)),windows)
  EXE_EXT := .exe
else
  EXE_EXT :=
endif

build-engine:
	cd engine && uv run pyinstaller aurotype-engine.spec --noconfirm
	mkdir -p src-tauri/binaries
	cp engine/dist/aurotype-engine/aurotype-engine$(EXE_EXT) src-tauri/binaries/aurotype-engine-$(TRIPLE)$(EXE_EXT)

build: build-engine
	bun run tauri build
