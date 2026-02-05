#!/bin/bash
#rustc --print target-list
#rustup target add x86_64-pc-windows-gnu
#rustup target add aarch64-linux-android

cd `dirname $0`
cd ..

for arch in $(echo x86_64-unknown-linux-musl aarch64-unknown-linux-musl);do
	cargo build --release --target $arch
	echo 'y' | cp target/$arch/release/link2lan target/release/link2lan-$arch
	chmod +x target/release/link2lan-$arch
done

arch=aarch64-linux-android
CC=/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang cargo build --release --target $arch
echo 'y' | cp target/$arch/release/link2lan target/release/link2lan-$arch
chmod +x target/release/link2lan-$arch

arch=x86_64-pc-windows-gnu
cargo build --release --target $arch
echo 'y' | cp target/$arch/release/link2lan.exe target/release/link2lan-${arch}.exe

cd target/release
sha256sum link2lan-* > sha256sum.txt
ls -l link2lan-*
