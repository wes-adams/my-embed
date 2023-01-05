#!/bin/bash

RELEASE_DIR="/home/wadams/Code/my-embed/target/thumbv6m-none-eabi/release"

cargo build --release
elf2uf2-rs $RELEASE_DIR/my-embed
picotool load -x $RELEASE_DIR/my-embed.uf2

