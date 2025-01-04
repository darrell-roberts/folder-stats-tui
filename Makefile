PLATFORM := $(shell uname)

all: build

clean-dist:
	rm -rf dist/

check:
	cargo clippy

build: check
	cargo build --release

linux-app-image: clean-dist build
	echo "Building linux app image"
	rm -rf dist/AppDir
	# create new AppDir
	linuxdeploy-x86_64.AppImage \
	   -e target/release/folder-stats-tui \
	   -d assets/folder-stats-tui.desktop \
	   -i assets/folder-stats-tui.svg \
	   --appdir dist/AppDir

	# Create app image
	linuxdeploy-x86_64.AppImage --appdir dist/AppDir --output appimage

.PHONY: all clean-dist check build linux-app-image
