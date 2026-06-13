RUST_CRATE := frontend/rust/retrofront-core
IOS_TARGET := aarch64-apple-ios
RUST_FEATURES := --features librashader-runtime

.PHONY: test rust linux-ui ios-rust xcodegen clean

test:
	cargo test --manifest-path $(RUST_CRATE)/Cargo.toml

rust:
	cargo build --manifest-path $(RUST_CRATE)/Cargo.toml --release $(RUST_FEATURES)

linux-ui: rust
	mkdir -p build/linux
	swiftc -import-objc-header platforms/linux/Sources/retrofront_bridge.h \
	  platforms/linux/Sources/main.swift \
	  -L ../target/release -lretrofront_core \
	  -Xlinker -rpath -Xlinker '$$ORIGIN' \
	  -o build/linux/retrofront-linux
	cp ../target/release/libretrofront_core.so build/linux/

ios-rust:
	rustup target add $(IOS_TARGET)
	cargo build --manifest-path $(RUST_CRATE)/Cargo.toml --release $(RUST_FEATURES) --target $(IOS_TARGET)
	mkdir -p lib/ios
	cp ../target/$(IOS_TARGET)/release/libretrofront_core.a lib/ios/libretrofront_core.a

xcodegen:
	xcodegen generate --spec project.yml

clean:
	cargo clean
	rm -rf .build build DerivedData Payload Retrofront.ipa Retrofront.xcodeproj lib/ios/libretrofront_core.a
