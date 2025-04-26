# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands
- Frontend development: `npm run dev`
- Backend development: `cargo tauri dev`
- Type checking: `npm run check`
- Build application: `npm run build`
- Frontend preview: `npm run preview`
- Run Rust tests: `cargo test` (in src-tauri directory)
- Run single Rust test: `cargo test test_name` (in src-tauri directory)

## Code Style Guidelines
- **TypeScript/Svelte**: Use TypeScript for type safety. Follow Svelte component structure.
- **Rust**: Follow Rust idioms, use Result for error handling, employ log macros.
- **Error Handling**: Use Result<T, E> in Rust, try/catch in TypeScript. Log errors appropriately.
- **Logging**: Use `logger.error/warn/info/debug/trace()` in frontend, `log::error!/warn!/info!` in backend.
- **Imports**: Group imports by source (std lib, external, internal). 
- **Naming**: camelCase for JS/TS, snake_case for Rust. Use descriptive names.
- **Components**: Follow existing patterns in components directory.
- **State Management**: Use Svelte stores for frontend state.