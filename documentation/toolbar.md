# Seelen UI Toolbar - Layouts and Customization

> **Warning:** Do not modify the files in the installation directory. These files are overwritten with each update. To make custom changes, please follow the next guide.

Seelen UI allows you to fully customize your desktop environment, including the toolbar. The toolbar, also known as the "placeholder," can be configured using YAML files and tailored to your needs with various dynamic elements. This document guides you through customizing and managing toolbar items, outlining the available scopes, and providing examples.

## Placeholders

The toolbar layout, referred to as the "placeholder," is defined in a YAML file that follows the [placeholder schema](https://github.com/Seelen-Inc/slu-lib/blob/master/gen/schemas/toolbar_items.schema.json). It can be personalized using Themes.

Example:

```yaml
left:
  - type: text
    template: concat("@", env.USERNAME)
    onClickV2: open(env.USERPROFILE)
    tooltip: '"Open user folder"'
    style:
      fontSize: 24
      fontWeight: bold
```

## Location of Placeholder Files

Placeholder files must be in `.yml` or `.yaml`. The file should be located in the following directory:

```text
C:\Users\{USER}\AppData\Roaming\com.seelen.seelen-ui\placeholders
└── YourPlaceholderFile.yml
```

**Note:** The file name is used as the identifier for the placeholder configuration.

### Details

- **style**: The `style` property follows the React `style` prop conventions. For more details, refer to the [React style documentation](https://reactjs.org/docs/dom-elements.html#style).
- **id**: The `id` field is used as the HTML element ID in the DOM, allowing you to apply specific styles via CSS. For general styling within the item, use the `style` property. For more advanced customizations, refer to the [themes documentation](./documentation/themes.md).

## How to Use and Edit Placeholders in Seelen UI

Seelen UI allows you to display dynamic information in the toolbar using placeholders. These placeholders are configured based on toolbar items, each with its own `scope` that provides access to various system properties and functions. Below is a guide on how to use them.

### Base Structure of a Toolbar Item

All toolbar items share a base structure that includes properties like `id`, `template`, `tooltip`, `badge`, `onClick`, `onClickV2`, and `style`, other specific properties are available for each item type, declared in the [placeholder schema](https://github.com/Seelen-Inc/slu-lib/blob/master/gen/schemas/toolbar_items.schema.json).

> **Deprecated**: The `onClick` property is deprecated and will be removed in future versions. Please use `onClickV2` instead.

### Code in YAML

In Seelen UI, `template`, `tooltip`, `badge`, and `onClickV2` function bodies are defined as code. This code is evaluated at runtime using the [mathjs](https://mathjs.org/) evaluate function, similar to how Conditional Layouts operate.

When writing literal strings in YAML, use double quotes:

```yaml
 tooltip: '"Open user folder"'
```

#### TextToolbarItem: The Base Scope

The `TextToolbarItem` serves as the foundation for all other items. The scope available to `TextToolbarItem` is also applicable to all other modules. This scope includes:

```ts
const icon: object; // All icons defined in React Icons
const env: object;  // All environment variables defined on the system

function getIcon(name: string, size: number = 16): string

function imgFromUrl(url: string, size: number = 16): string
function imgFromPath(path: string, size: number = 16): string
function imgFromExe(exe_path: string, size: number = 16): string

/**
 * The next function is used to get a specific text by the used language.
 * As example: `t("placeholder.notifications")`
*/
function t(path: string): string
```

### Available Scopes

Each type of item has a specific scope that extends the properties and functions available. Below are the scopes for each item type.

#### GenericToolbarItem Scope

This scope includes information about the currently focused window:

```ts
const window: {
    name: string;
    title: string;
    exe: string | null;
};
```

#### DateToolbarItem Scope

This scope allows you to display the formatted date and configure the update interval:

```ts
const date: string; // The formatted date
```

#### PowerToolbarItem Scope

This scope includes information about power status and battery details:

```ts
interface PowerStatus {
    acLineStatus: number;
    batteryFlag: number;
    batteryLifePercent: number;
    systemStatusFlag: number;
    batteryLifeTime: number;
    batteryFullLifeTime: number;
}

interface Battery {
    vendor: string | null;
    model: string | null;
    serialNumber: string | null;
    technology: string;
    state: string;
    capacity: number;
    temperature: number | null;
    percentage: number;
    cycleCount: number | null;
    smartCharging: boolean;
    energy: number;
    energyFull: number;
    energyFullDesign: number;
    energyRate: number;
    voltage: number;
    timeToFull: number | null;
    timeToEmpty: number | null;
}

const power: PowerStatus;
const batteries: Battery[];
const battery: Battery | null;
```

#### NetworkToolbarItem Scope

This scope provides details about network interfaces:

```ts
interface NetworkInterface {
    name: string;
    description: string;
    status: 'up' | 'down';
    dnsSuffix: string;
    type: string;
    gateway: string | null;
    mac: string;
    ipv4: string | null;
    ipv6: string | null;
}

const online: boolean;
const interfaces: NetworkInterface[];
const usingInterface: NetworkInterface | null;
```

#### MediaToolbarItem Scope

This scope includes information about media sessions and volume control:

```ts
const volume: number; // Output master volume from 0 to 1
const isMuted: boolean; // Output master volume is muted
const inputVolume: number; // Input master volume from 0 to 1
const inputIsMuted: boolean; // Input master volume is muted

interface MediaSession {
    umid: string;
    title: string;
    author: string;
    thumbnail: string | null; // Path to temporary media session image
    playing: boolean;
    default: boolean;
    owner: {
        name: string;
    };
}

const mediaSession: MediaSession | null;
```

#### NotificationsToolbarItem Scope

This scope provides information about notifications:

```ts
const count: number; // Number of notifications
```

#### Other Toolbar Item Scopes

- **TrayToolbarItem Scope**: This module does not expand the scope of the item.
- **DeviceToolbarItem Scope**: This module does not expand the scope of the item.
- **SettingsToolbarItem Scope**: This module does not expand the scope of the item.
- **WorkspaceToolbarItem Scope**: This module does not expand the scope of the item.
