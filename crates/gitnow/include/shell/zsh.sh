function git-now {
  choice=$(gitnow  "$@" --no-shell)
    if [[ $? -ne 0 ]]; then
    return $?
  fi
  
  cd "$(echo "$choice" | tail --lines 1)"
}

function gn {
  git-now "$@"
  if [[ $? -ne 0 ]]; then
    return $?
  fi
}
