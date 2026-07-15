# Godot Hub

[![GitHub Release](https://img.shields.io/github/v/release/darwinbillian/godot-hub?style=flat-square)](https://github.com/darwinbillian/godot-hub/releases)
[![GitHub License](https://img.shields.io/github/license/darwinbillian/godot-hub?style=flat-square)](./LICENSE)
[![GitHub last commit](https://img.shields.io/github/last-commit/darwinbillian/godot-hub?style=flat-square)](https://github.com/darwinbillian/godot-hub/commits/main/)

Godot Hub is a desktop application for managing multiple versions of the [Godot Engine](https://godotengine.org/).

![Screenshot](./assets/screenshot.png)

## Features

- **Version Management:** Download, manage, and launch different versions of the Godot Engine.
- **Lightweight and Fast:** Powered by Tauri.

## Installation

Download the [latest](https://github.com/darwinbillian/godot-hub/releases/latest) installer and run it.

## Development

Install Node.js and Rust, then run the application:

```sh
git clone https://github.com/darwinbillian/godot-hub.git
cd godot-hub
npm ci
npm run tauri dev
```

To build the application:

```sh
npm run tauri build
```

## License

This project is licensed under the [GNU General Public License v3.0](./LICENSE).
