# Jot

A minimal note-taking app for quick thoughts and ideas.

## Overview

Jot is a lightweight, minimalist note-taking application built with Tauri (Rust) and Svelte. Inspired by apps like Tot, Jot provides a simple way to organize your thoughts across seven color-coded tabs while maintaining a small footprint (under 10MB) and lightning-fast startup times (under 1 second).

## Features

- **Seven Color-Coded Tabs**: Quickly navigate between different notes with visually distinct tabs
- **Partial Markdown Support**: Simple formatting including headers, bold, italic, lists, and links
- **Dual View Modes**: Switch between edit and preview modes
- **Text Statistics**: Automatic word, character, and line counting
- **Dark/Light Theme**: Comfortable writing experience in any lighting environment  
- **Persistent Storage**: Automatic saving of your notes
- **Keyboard Shortcuts**: Efficient navigation and text manipulation
- **Cross-Platform**: Works on Windows and macOS

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.60 or later
- [Node.js](https://nodejs.org/) 14 or later
- Tauri CLI: `cargo install tauri-cli`

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

| Shortcut    | Action                      |
|-------------|----------------------------- |
| Ctrl+1-7    | Switch to tab 1-7           |
| Ctrl+E      | Toggle edit/preview mode    |
| Ctrl+D      | Toggle dark/light theme     |
| Ctrl+B      | Bold text                   |
| Ctrl+I      | Italic text                 |
| Ctrl+T      | Insert timestamp            |
| Tab         | Insert indentation          |
| Shift+Tab   | Remove indentation          |

### Markdown Support

Jot supports common markdown syntax:

- `**bold text**` - Bold text
- `*italic text*` - Italic text
- `- item` - List item
- `[text](url)` - Link

## Project Structure

```
jot/
├── src/                # Rust backend code
│   ├── lib.rs          # Tauri commands implementation
│   └── main.rs         # Application entry point
├── src-tauri/          # Tauri configuration
├── src/                # Svelte frontend
│   ├── lib/
│   │   ├── components/ # UI components
│   │   ├── stores/     # Svelte stores
│   │   └── utils/      # Utility functions
│   └── app.html        # App entry point
├── public/             # Static assets
└── package.json        # Project dependencies
```

## Technical Details

- **Frontend**: Svelte, TypeScript
- **Backend**: Rust, Tauri
- **Storage**: Local file system for persistence
- **Bundle Size**: <10MB
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

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by [Tot](https://tot.rocks/) and other minimal note-taking apps
- Built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/)
- Icon designs from [FontAwesome](https://fontawesome.com/)
- App Icon [Sticky note icons created by Muhammad_Usman - Flaticon](https://www.flaticon.com/free-icons/sticky-note)