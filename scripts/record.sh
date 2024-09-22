#!/usr/bin/env zsh

set -e

# Loop through each file in the folder
for file in "vhs"/*; do
  # Check if it is a file (not a directory)
  if [[ -f "$file" ]]; then
    echo "Recording: $file"

    vhs "./$file"
  fi
done
