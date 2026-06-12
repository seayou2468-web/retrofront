.PHONY: all rust swift swift-core test clean \
        linux-ui linux-core linux-appimage \
        frontend-assets ios-rust xcodegen ios-device-build ios-info \
        strip-linux strip-ios

# =========================
# Default build
# =========================
all: rust swift

# =========================
# Rust core (shared)
# =========================
rust:
	cargo build --manifest-path Cargo.toml --release

# iOS専用Rustビルド
ios-rust:
	./scripts/build-rust-ios.sh

# =========================
# Swift builds (macOS / Linux)
# =========================

# GUIなし軽量ビルド（テスト・CI用）
swift-core: rust
	RETROFRONT_DISABLE_LINUX_GUI=1 swift build

# フルLinux GUIビルド
swift: rust
	swift build -c release

# =========================
# Linux builds
# =========================

# GUI必須チェック付きビルド
linux-ui: rust
	@pkg-config --exists libadwaita-1 gtksourceview-5 || \
	(echo "Missing dependencies: install libadwaita-1-dev and libgtksourceview-5-dev" >&2; exit 1)
	swift build -c release --product retrofront-linux

# GUIなしコアビルド（ヘッドレス or CI用）
linux-core: rust
	RETROFRONT_DISABLE_LINUX_GUI=1 swift build -c release --product retrofront-linux

# AppImage生成
linux-appimage: frontend-assets linux-ui strip-linux
	./scripts/build-linux-appimage.sh

# =========================
# Frontend assets
# =========================
frontend-assets:
	./scripts/fetch-frontend-assets.sh apps/iOS/Resources

# =========================
# iOS build (XcodeGen)
# =========================
xcodegen:
	./scripts/generate-xcodeproj.sh

ios-device-build: ios-rust xcodegen
	xcodebuild \
		-project Retrofront.xcodeproj \
		-scheme RetrofrontiOS \
		-destination 'generic/platform=iOS' \
		-configuration Debug \
		CODE_SIGNING_ALLOWED=NO \
		build

ios-info:
	@echo "Rust iOS build:"
	@echo "  cargo build --release --target aarch64-apple-ios"
	@echo ""
	@echo "Then link libretrofront_core.a into Xcode project"

# =========================
# Tests
# =========================
test: rust
	cargo test --manifest-path Cargo.toml --release
	RETROFRONT_DISABLE_LINUX_GUI=1 swift test

# =========================
# Cleanup
# =========================
clean:
	cargo clean --manifest-path Cargo.toml
	swift package clean

# =========================
# Size optimization
# =========================

# Linuxバイナリサイズ削減
strip-linux:
	@echo "Stripping Linux binaries..."
	@strip target/release/retrofront-linux 2>/dev/null || true

# iOS用（必要なら）
strip-ios:
	@echo "Stripping iOS Rust static library..."
	@strip target/aarch64-apple-ios/release/libretrofront_core.a 2>/dev/null || true
