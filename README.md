# Git Now

Git Now is a utility for easily navigating git projects from common upstream providers. Search, Download, and Enter projects as quickly as you can type.

![example gif](./assets/gifs/example.gif)

## Installation

```bash
cargo (b)install gitnow
```

## Reasoning

How many steps do you normally do to download a project?

1. Navigate to github.com
2. Search in your org for the project
3. Find the clone url
4. Navigate to your local github repositories path
5. Git clone `<project>` 
6. Enter new project directory

A power user can of course use `gh repo clone` to skip a few steps.

With gitnow

1. `git now`
2. Enter parts of the project name and press enter
3. Your project is automatically downloaded if it doesn't exist in an opinionated path dir, and move you there. 
