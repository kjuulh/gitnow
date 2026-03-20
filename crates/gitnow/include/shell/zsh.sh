function git-now {
  # Run an update in the background
  (
    nohup gitnow update > /dev/null 2>&1 &
  )

  # Create a temporary chooser file
  local chooser_file
  chooser_file="$(mktemp)"

  # Run gitnow with the chooser file
  GITNOW_CHOOSER_FILE="$chooser_file" gitnow "$@"
  local exit_code=$?

  if [[ $exit_code -ne 0 ]]; then
    rm -f "$chooser_file"
    return $exit_code
  fi

  # If the chooser file has content, cd to the chosen path
  if [[ -s "$chooser_file" ]]; then
    local target
    target="$(cat "$chooser_file")"
    rm -f "$chooser_file"
    cd "$target"
  else
    rm -f "$chooser_file"
  fi
}

function gn {
  git-now "$@"
}
