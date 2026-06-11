.PHONY: all rust flutter-test flutter-linux flutter-ios test clean linux-ui linux-appimage frontend-assets ios-rust xcodegen ios-device-build ios-info

FLUTTER ?= flutter
FLUTTER_DIR := flutter

all: rust flutter-test

rust:
	cargo build --manifest-path Cargo.toml --release -p retrofront-core

flutter-test:
	cd $(FLUTTER_DIR) && $(FLUTTER) test

test: rust
	cargo test --manifest-path Cargo.toml -p retrofront-core
	@if command -v $(FLUTTER) >/dev/null 2>&1; then \
		$(MAKE) flutter-test; \
	else \
		echo "flutter not found; skipping Flutter tests"; \
	fi

flutter-linux: rust
	cd $(FLUTTER_DIR) && $(FLUTTER) build linux --release

flutter-ios: ios-rust frontend-assets
	cd $(FLUTTER_DIR) && $(FLUTTER) build ios --release --no-codesign

linux-ui: flutter-linux

linux-appimage: frontend-assets flutter-linux
	../scripts/build-linux-appimage.sh

frontend-assets:
	../scripts/fetch-frontend-assets.sh flutter/assets/retroarch

ios-rust:
	../scripts/build-rust-ios.sh

xcodegen:
	../scripts/generate-xcodeproj.sh

ios-device-build: ios-rust xcodegen
	cd $(FLUTTER_DIR) && $(FLUTTER) build ios --release --no-codesign

clean:
	cargo clean --manifest-path Cargo.toml
	@if command -v $(FLUTTER) >/dev/null 2>&1; then cd $(FLUTTER_DIR) && $(FLUTTER) clean; fi

# Build the native Retrofront FFI library for an iOS target after installing it:
#   rustup target add aarch64-apple-ios
#   make ios-info IOS_TARGET=aarch64-apple-ios
ios-info:
	@echo "Build the Rust FFI core with: cargo build --manifest-path Cargo.toml --release -p retrofront-core --target $${IOS_TARGET:-aarch64-apple-ios}"
	@echo "Flutter uses flutter/lib/main.dart for both iOS and Linux; no Slint UI is built."
