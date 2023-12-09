# Folder stats
A simple Tui that shows folder stats as two bar charts, one for aggregate folder file size and one for aggregate file counts.

* Toggle .ignore/.gitignore support. Enabled by default.
* Filter folders by file name.
* Filter folders by file name extension.
* Set the folder depth to view (keys 1-8 in Tui).
* Sort folders by file size or file count (keys "s" for size and "c" for count in Tui).
* Key "q" to quit Tui.

## Options
```
Command line arguments

Usage: folder-stats-tui [OPTIONS]

Options:
  -p, --path <ROOT_PATH>              Folder to scan. [default: .]
  -d, --depth <DEPTH>                 Folder depth to see in Tui [default: 1]
  -f, --filter <FILTER>               Filter files that contain text
  -e, --extension <EXTENSION_FILTER>  Filter by file extension. Ex: -e rs
  -i, --no-ignores                    Disable .ignore, .gitignore filtering
      --show-hidden                   Disable hidden file filtering
  -h, --help                          Print help
```

<img width="1234" alt="image" src="https://github.com/darrell-roberts/folder-stats-tui/assets/33698065/e89e44ac-1ea3-47df-8d03-93a54da27376">

<img width="1234" alt="image" src="https://github.com/darrell-roberts/folder-stats-tui/assets/33698065/a20c2137-0469-4abe-ac6c-943da2dffab1">

