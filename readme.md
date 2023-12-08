# Folder stats
A simple Tui that shows folder stats as two bar charts, one for aggregate folder file size and one for aggregate file counts.

* Toggle .ignore/.gitnore support. Enabled by default.
* Filter folders by filename
* Filter folders by file name extension.
* Set the folder depth to view (keys 1-8 in Tui)
* Sort folders by file size or file count (keys "s" for size and "c" for count in Tui)
* Key "q" to quit Tui

## Options
```
Usage: folder-stats-tui [OPTIONS]

Options:
  -p, --path <ROOT_PATH>              Folder to scan. [default: .]
  -d, --depth <DEPTH>                 Folder depth to see in Tui [default: 1]
  -f, --filter <FILTER>               Filter files that contain text
  -e, --extension <EXTENSION_FILTER>  Filter by file extension. Ex: -e rs
  -i, --no-ignores                    Disable .ignore, .gitignore filtering
  -h, --help                          Print help
```

<img width="1344" alt="image" src="https://github.com/darrell-roberts/folder-stats-tui/assets/33698065/80a7c528-e589-4705-9120-6b64be17f348">
