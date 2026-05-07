# Jot

A minimal note-taking app for quick thoughts and ideas.

## Overview

Jot is a lightweight, minimalist note-taking application built with Tauri (Rust) and Svelte. Inspired by apps like Tot, Jot provides a simple way to organize your thoughts across seven color-coded tabs while maintaining a small footprint and lightning-fast startup times.

## Features

- **Seven Color-Coded Tabs**: Quickly navigate between different notes with visually distinct tabs
- **WYSIWYG Markdown Editor**: Real-time markdown rendering while you type
- **Text Statistics**: Automatic word, character, and line counting
- **Dark/Light Theme**: Comfortable writing experience in any lighting environment
- **Persistent Storage**: Automatic saving of your notes with backup support
- **Keyboard Shortcuts**: Efficient navigation and text manipulation
- **Global Shortcut**: Show/hide window from anywhere
- **Cross-Platform**: Works on Windows and macOS

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later
- [Node.js](https://nodejs.org/) 20 or later
- Tauri CLI: `cargo install tauri-cli --version "^2"`

### Building from Source

1. Clone the repository:

   ```
   git clone https://github.com/afry23/jot.git
   cd jot
   ```

2. Install dependencies:

   ```
   npm install
   ```

3. Run in development mode:

   ```
   npm run tauri dev
   ```

4. Build for production:

   ```
   npm run tauri build
   ```

## Usage

### Keyboard Shortcuts

| Shortcut       | Action                      |
|----------------|-----------------------------|
| Ctrl+1-7       | Switch to tab 1-7           |
| Ctrl+D         | Toggle dark/light theme     |
| Ctrl+B         | Bold text                   |
| Ctrl+I         | Italic text                 |
| Ctrl+T         | Insert timestamp            |
| Ctrl+L         | Insert unordered list       |
| Ctrl+O         | Insert ordered list         |
| Ctrl+H         | Insert link                 |
| Ctrl+Z         | Undo                        |
| Ctrl+Y         | Redo                        |
| Tab            | Insert indentation          |
| Shift+Tab      | Remove indentation          |
| Ctrl+Shift+J   | Hide/Show window            |
| Ctrl+Shift+B   | Create backup               |

### Markdown Support

Jot supports common markdown syntax with live preview:

- `**bold text**` - Bold text
- `*italic text*` - Italic text
- `- item` - List item
- `1. item` - Ordered list item
- `[text](url)` - Link
- Headings with `#`, `##`, etc.

## Project Structure

```
jot/
├── src-tauri/          # Rust backend + Tauri configuration
│   ├── src/
│   │   ├── lib.rs      # Tauri commands implementation
│   │   └── main.rs     # Application entry point
│   └── tauri.conf.json
├── src/                # Svelte frontend
│   ├── lib/
│   │   ├── components/ # UI components
│   │   ├── stores/     # Svelte stores
│   │   └── utils/      # Utility functions
│   └── app.html        # App entry point
└── package.json        # Project dependencies
```

## Technical Details

- **Frontend**: Svelte, TypeScript
- **Backend**: Rust, Tauri v2
- **Storage**: Local file system for persistence
- **Bundle Size**: <15MB
- **Startup Time**: <1 second
- **Dependencies**: Minimal external dependencies to maintain small footprint

## Design Philosophy

Jot is designed with the following principles in mind:

1. **Minimalism**: Focus on content, not features
2. **Speed**: Fast startup, responsive UI
3. **Reliability**: Your notes are always saved
4. **Accessibility**: Comfortable in any environment with theming support
5. **Efficiency**: Keyboard-driven workflow

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the GNU GPL Version 3 License - see the LICENSE file for details.

## Acknowledgments

- Inspired by [Tot](https://tot.rocks/) and other minimal note-taking apps
- Built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/)
- Icon designs from [FontAwesome](https://fontawesome.com/)
- App Icon [Sticky note icons created by Muhammad_Usman - Flaticon](https://www.flaticon.com/free-icons/sticky-note)
