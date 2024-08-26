# Welcome to the Seelen UI Project

Welcome to the Seelen UI project! This guide will help you get started with the codebase and understand its structure.

## Languages Used
This project utilizes the following languages:
- **Rust**
- **TypeScript**
- **PowerShell** (in special cases)

## Getting Started
To run this project, follow these steps:

1. [Install Rust](https://www.rust-lang.org/tools/install).
2. Run the following commands:

```bash
npm install && npm run dev
```

This will set up the project similarly to any other Node.js project, with the added step of installing Rust first.

## Architecture

### Views Architecture

The `src\apps` folder contains views that follow Hexagonal Architecture. Each folder in `src\apps` represents a view (excluding shared). These views are independent web pages bundled with `esbuild`. While any technology or library can be used in a view, most are based on `React` and `Redux`.

#### Shared Folder
Following Hexagonal Architecture, the `shared` folder contains utilities, schemas, and other shared resources used across multiple views.

### Background Architecture

The `src\background` folder does not follow a specific architecture but is based on Events Architecture.

### Hierarchical Locking Order

To prevent deadlocks in the application, all threads must follow the "Hierarchical Locking Order" when acquiring resources:

1. **CLI**: Acquire any locks related to the command-line interface first.
2. **DATA**: Next, acquire locks related to data access or shared data structures.
3. **EVENT**: Finally, acquire locks related to hook or event management.

This order must be respected in all threads to avoid circular wait conditions and ensure safe concurrency.
