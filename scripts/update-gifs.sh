#!/usr/bin/env zsh

rm -r assets/gifs

set -e

cuddle x record
mkdir -p assets/gifs
mv target/vhs/* assets/gifs
