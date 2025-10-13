# Svelte UI Components Library

This library contains shared Svelte components and utilities for Seelen UI.

## Components

### Icon

A Svelte component for rendering SVG icons with support for sizing and coloring.

#### Usage

```svelte
<script>
  import { Icon } from "$lib/ui/svelte/components/Icon";
</script>

<Icon iconName="FiSettings" size="24px" color="red" />
<Icon iconName="BiPlus" size={32} />
```

#### Props

- `iconName` (IconName, required): The name of the icon from react-icons
- `size` (string | number, optional): Size of the icon (defaults to 1rem)
- `color` (string, optional): Color of the icon
- `class` (string, optional): Additional CSS classes
- `style` (string, optional): Additional inline styles

### InlineSVG

A lower-level component for rendering inline SVG content from a URL.

#### Usage

```svelte
<script>
  import { InlineSVG } from "$lib/ui/svelte/components/Icon";
</script>

<InlineSVG src="/path/to/icon.svg" class="my-icon" />
```

## Utilities

### isDarkModeEnabled

Checks if the system is using dark mode.

```typescript
import { isDarkModeEnabled } from "$lib/ui/svelte/utils/styles";

const isDark = isDarkModeEnabled();
```

## Directory Structure

```
libs/ui/svelte/
├── components/
│   └── Icon/
│       ├── Icon.svelte
│       ├── InlineSVG.svelte
│       └── index.ts
├── utils/
│   └── styles.ts
└── README.md
```
