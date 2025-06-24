# Erabooru File Watcher

A small cross-platform desktop application built with [Tauri](https://tauri.app/) and [SvelteKit](https://kit.svelte.dev/) for automatically uploading new images or videos from a folder to an EraBooru compatible server.

## Features

- Monitors a single folder for newly created media files.
- Automatically uploads images and videos to the configured server.
- Settings (watched folder and server address) are persisted locally using the Tauri store plugin.
- Simple Svelte based interface to start/stop watching and manage settings.

## Prerequisites

- [Node.js](https://nodejs.org/) and npm
- [Rust](https://www.rust-lang.org/tools/install)
- The [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites/#installing-the-tauri-cli)

## Getting Started

Install JavaScript dependencies:

```bash
npm install
```

Run the application in development mode:

```bash
npm run tauri dev
```

This will launch the SvelteKit frontend and the Tauri shell. Changes in the source files cause the app to reload automatically.

To run type checks use:

```bash
npm run check
```

## Building

Create a production build of the application:

```bash
npm run tauri build
```

The binaries will be available in the `src-tauri/target` directory for your platform.

## Usage

1. Start the application.
2. Use **Select** to choose the folder you want to monitor.
3. Enter the base URL of your EraBooru server (e.g. `http://localhost:8000`).
4. Click **Save** to persist the settings.
5. Click **Run** to begin watching. New images and videos placed in the folder will be uploaded automatically.

The application calculates a content hash for each file and uses the `/api/media/upload-url` endpoint to obtain a pre-signed upload URL from the server before uploading.

## License

This project is licensed under the terms of the MIT license. See [LICENSE](LICENSE) for details.

