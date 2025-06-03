#!/bin/bash
#rustup target add x86_64-pc-windows-gnu
#rustup target add aarch64-linux-android
#yay -Ss android-aarch64-openssl

cd `dirname $0`
cd ..
OPENSSL_VERSION=3.5.0
arch=x86_64-unknown-linux-musl
OPENSSL_STATIC=1 OPENSSL_DIR=/opt/musl/openssl-${OPENSSL_VERSION} cargo build --release --target x86_64-unknown-linux-musl
echo 'y' | cp target/$arch/release/link2lan target/release/link2lan-$arch

arch=aarch64-linux-android
CC=/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang OPENSSL_STATIC=1 OPENSSL_DIR=/opt/android-libs/aarch64 cargo build --release --target aarch64-linux-android
echo 'y' | cp target/$arch/release/link2lan target/release/link2lan-$arch

arch=x86_64-pc-windows-gnu
cargo build --release --target $arch
echo 'y' | cp target/$arch/release/link2lan.exe target/release/link2lan-${arch}.exe

cd target/release
sha256sum link2lan-* > sha256sum.txt
ls -l link2lan-*
