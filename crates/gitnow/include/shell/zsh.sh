function git-now {
  # Run an update in the background
  (
    nohup gitnow update > /dev/null 2>&1 &
  )

  # Find the repository of choice
  choice=$(gitnow  "$@" --no-shell)
    if [[ $? -ne 0 ]]; then
    return $?
  fi

  # Enter local repository path
  cd "$(echo "$choice" | tail --lines 1)"
}

function gn {
  git-now "$@"
  if [[ $? -ne 0 ]]; then
    return $?
  fi
}
