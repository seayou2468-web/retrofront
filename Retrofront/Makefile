.PHONY: all rust swift test clean ios-info

all: rust swift

rust:
	cargo build --manifest-path Cargo.toml --release

swift: rust
	swift build

test: rust
	cargo test --manifest-path Cargo.toml
	swift test

clean:
	cargo clean --manifest-path Cargo.toml
	swift package clean

# Build the Rust static library for an iOS target after installing the target, e.g.:
#   rustup target add aarch64-apple-ios
#   make ios-info IOS_TARGET=aarch64-apple-ios
ios-info:
	@echo "Build iOS Rust core with: cargo build --manifest-path Cargo.toml --release --target $${IOS_TARGET:-aarch64-apple-ios}"
	@echo "Then link libretrofront_core.a into your iOS app target and import RetrofrontSwift."
