# Folder stats
A simple Tui that shows folder stats

* Toggle .ignore/.gitnore support. Enabled by default.
* Filter results by filename
* Filter results by file name extension.
* Set the folder depth to view in results (keys 1-5 in Tui)
* Sort results by file size or file count (keys "s" for size and "c" for count in Tui)
* Key "q" to quit Tui

## Options
```
Usage: folder-stats-tui [OPTIONS]

Options:
  -p, --path <ROOT_PATH>              [default: .]
  -d, --depth <DEPTH>                 [default: 1]
  -f, --filter <FILTER>
  -e, --extension <EXTENSION_FILTER>
  -i, --no-ignores
  -h, --help                          Print help
```

<img width="1344" alt="image" src="https://github.com/darrell-roberts/folder-stats-tui/assets/33698065/80a7c528-e589-4705-9120-6b64be17f348">
