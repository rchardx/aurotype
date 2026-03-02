.PHONY: dev engine-dev setup build-engine sign-engine build

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
	cp engine/dist/aurotype-engine$(EXE_EXT) src-tauri/binaries/aurotype-engine-$(TRIPLE)$(EXE_EXT)

# macOS code signing for release distribution
# Set APPLE_SIGNING_IDENTITY to enable (e.g., "Developer ID Application: Your Name (TEAM_ID)")
# Required env vars for notarization (used by Tauri automatically during tauri build):
#   APPLE_SIGNING_IDENTITY  - signing certificate identity string
#   APPLE_CERTIFICATE       - base64-encoded .p12 certificate (for CI)
#   APPLE_CERTIFICATE_PASSWORD - .p12 certificate password (for CI)
#   APPLE_API_ISSUER        - App Store Connect API issuer UUID
#   APPLE_API_KEY           - App Store Connect API key ID
#   APPLE_API_KEY_PATH      - path to AuthKey .p8 file
#
# Release build flow:
#   export APPLE_SIGNING_IDENTITY="Developer ID Application: ..."
#   make build
sign-engine:
ifeq ($(shell uname -s),Darwin)
ifdef APPLE_SIGNING_IDENTITY
	codesign --force --sign "$(APPLE_SIGNING_IDENTITY)" --options runtime \
		--entitlements src-tauri/Entitlements.plist \
		src-tauri/binaries/aurotype-engine-$(TRIPLE)$(EXE_EXT)
endif
endif

build: build-engine sign-engine
	bun run tauri build
