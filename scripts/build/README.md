# Build System

This directory contains the modular build system for Seelen UI applications.

## Structure

```
scripts/build/
├── README.md           # This file
├── config.ts           # Shared configuration and argument parsing
├── types.ts            # TypeScript type definitions
├── server.ts           # Development server
├── builders/           # Framework-specific builders
│   ├── react.ts        # React/Preact build configuration
│   └── svelte.ts       # Svelte build configuration
├── plugins/            # Custom esbuild plugins
│   └── index.ts        # Copy public files plugin
└── steps/              # Build steps
    ├── cleanup.ts      # Clean dist directory
    ├── discover.ts     # Discover entry points
    └── icons.ts        # Extract icons from react-icons
```

## Build Process

The build process is orchestrated by `scripts/build.ts` and follows these steps:

1. **Parse Arguments** - Parse command-line flags (`--production`, `--serve`)
2. **Extract Icons** - Extract SVG icons from react-icons package (once)
3. **Clean Dist** - Remove old build artifacts (preserves icons)
4. **Discover Entry Points** - Find all application entry points
5. **Build React** - Bundle React/Preact applications
6. **Build Svelte** - Bundle Svelte applications
7. **Start Server** - Start development server (if `--serve` flag)

## Framework Support

### React/Preact

- Entry point: `src/ui/{app}/index.tsx`
- Uses Preact as a drop-in replacement for React
- Supports CSS Modules with camelCase conversion
- Configuration: `builders/react.ts`

### Svelte

- Entry point: `src/ui/{app}/index.svelte`
- Requires Svelte esbuild plugin (to be configured)
- Configuration: `builders/svelte.ts`

### Vanilla TypeScript

- Entry point: `src/ui/{app}/index.ts`
- Bundled with React build configuration
- No framework overhead

## Adding New Frameworks

To add support for a new framework:

1. Create a new builder in `builders/{framework}.ts`
2. Implement the build configuration and build function
3. Update `steps/discover.ts` to detect the new framework
4. Import and call the builder in `scripts/build.ts`

## Configuration

### Build Options

- `--production` - Enable production mode (minification, no sourcemaps)
- `--serve` - Start development server with watch mode

### Constants

Defined in `config.ts`:

- `DEV_SERVER_PORT` - Development server port (3579)
- `DIST_DIR` - Output directory (./dist)
- `ICONS_DIR` - Icons directory (./dist/icons)
- `UI_DIR` - UI source directory (src/ui)

## Custom Plugins

### Copy Public Plugin

Defined in `plugins/index.ts`

- Copies `public/` folders from each app to the dist directory
- Reorganizes nested output folders to root level
- Cleans up temporary directories

## Development

To run the build in development mode:

```bash
npm run build:ui
```

To run with dev server:

```bash
npm run build:ui -- --serve
```

To run in production mode:

```bash
npm run build:ui -- --production
```
