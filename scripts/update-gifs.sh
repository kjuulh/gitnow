#!/usr/bin/env zsh

rm -r assets/gifs

set -e

cargo build --features example && clear

cuddle x record
mkdir -p assets/gifs
mv target/vhs/* assets/gifs
