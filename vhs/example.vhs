Output "target/vhs/example.gif"
Set Theme "Dracula"
Set Width 1200
Set Height 1000
Hide
Type "cargo build --features example && clear"
Enter
Sleep 1s
Type "./target/debug/gitnow --no-cache"
Enter
Show
Sleep 2s
Type@500ms "bevy"
Sleep 1s
Enter
Sleep 5s
