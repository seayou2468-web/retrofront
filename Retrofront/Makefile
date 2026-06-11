.PHONY: all rust swift swift-core test clean linux-ui ios-rust xcodegen ios-device-build ios-info

all: rust swift

rust:
	cargo build --manifest-path Cargo.toml --release

swift: rust
	swift build

# Build/test non-GUI Swift targets without requiring libadwaita headers.
swift-core: rust
	RETROFRONT_DISABLE_LINUX_GUI=1 swift build

test: rust
	cargo test --manifest-path Cargo.toml
	RETROFRONT_DISABLE_LINUX_GUI=1 swift test

linux-ui: rust
	@pkg-config --exists libadwaita-1 gtksourceview-5 || (echo "Missing Linux GUI development packages. Install libadwaita-1-dev and libgtksourceview-5-dev, then retry." >&2; exit 1)
	swift build --product retrofront-linux

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
	@echo "Build iOS Rust core with: cargo build --manifest-path Cargo.toml --release --target $${IOS_TARGET:-aarch64-apple-ios}"
	@echo "Then link libretrofront_core.a into your iOS app target and import RetrofrontSwift."
