#!/bin/sh

TARGETS=("x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu")
BIN_NAME="ebay_authd"
UPX_FLAGS="--best --lzma"

rm -rf releases/
mkdir releases/

for target in ${TARGETS[@]}; do
    cargo build --target=$target --release

    if [ -f target/$target/release/$BIN_NAME ]; then
        mv target/$target/release/$BIN_NAME releases/$BIN_NAME-$target

        if [ -x "$(command -v upx)" ]; then
            cp releases/$BIN_NAME-$target releases/$BIN_NAME-$target-compressed

            if ! upx $UPX_FLAGS releases/$BIN_NAME-$target-compressed; then
                rm releases/$BIN_NAME-$target-compressed
            fi
        fi
    fi

    if [ -f target/$target/release/$BIN_NAME.exe ]; then
        mv target/$target/release/$BIN_NAME.exe releases/$BIN_NAME-$target.exe

        if [ -x "$(command -v upx)" ]; then
            cp releases/$BIN_NAME-$target.exe releases/$BIN_NAME-$target-compressed.exe

            if ! upx $UPX_FLAGS releases/$BIN_NAME-$target-compressed.exe; then
                rm releases/$BIN_NAME-$target-compressed.exe
            fi
        fi
    fi
done