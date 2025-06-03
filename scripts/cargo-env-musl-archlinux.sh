OPENSSL_VERSION=3.5.0

sudo pacman -Syu musl kernel-headers-musl
rustup target add --toolchain stable x86_64-unknown-linux-musl

wget https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz
tar -zxvf openssl-${OPENSSL_VERSION}.tar.gz
cd openssl-${OPENSSL_VERSION}
CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/opt/musl/openssl-${OPENSSL_VERSION}
make -j `nproc`

make install
#OPENSSL_STATIC=1 OPENSSL_DIR=/opt/musl/openssl-${OPENSSL_VERSION} cargo build --release --target x86_64-unknown-linux-musl

#Reference: https://gist.github.com/zhangw/221dea4264539910a84550ad1acb52ad