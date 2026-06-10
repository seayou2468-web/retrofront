.PHONY: all rust swift test clean linux-ui linux-appimage frontend-assets ios-rust xcodegen ios-device-build ios-info

all: rust swift

rust:
	cargo build --manifest-path Cargo.toml --release

swift: rust
	swift build

test: rust
	cargo test --manifest-path Cargo.toml
	swift test

linux-ui:
	cargo build --manifest-path Cargo.toml --release -p retrofront-ui --bin retrofront

linux-appimage: frontend-assets
	./scripts/build-linux-appimage.sh

frontend-assets:
	./scripts/fetch-frontend-assets.sh apps/iOS/Resources

ios-rust:
	./scripts/build-rust-ios.sh

xcodegen:
	./scripts/generate-xcodeproj.sh

ios-device-build: ios-rust xcodegen
	xcodebuild -project Retrofront.xcodeproj -scheme RetrofrontiOS -destination 'generic/platform=iOS' -configuration Debug CODE_SIGNING_ALLOWED=NO build

clean:
	cargo clean --manifest-path Cargo.toml
	swift package clean

# Build the Rust static library for an iOS target after installing the target, e.g.:
#   rustup target add aarch64-apple-ios
#   make ios-info IOS_TARGET=aarch64-apple-ios
ios-info:
	@echo "Build iOS Slint UI with: cargo build --manifest-path Cargo.toml --release -p retrofront-ui --lib --target $${IOS_TARGET:-aarch64-apple-ios}"
	@echo "Then link libretrofront_ui.a into the iOS app target; SwiftUI is not used."
