# Seelen UI - Toolbar

## Placeholders
The toolbar layout, also known as "placeholder," can be defined in a YAML file, adhering to the [placeholder schema](./schemas/placeholder.schema.json) and customized using Themes.

To create a Toolbar module, follow this structure:

```yaml
left:
  - type: text
    template: concat("@", env.USERNAME)
    onClick: open -> env.USERPROFILE
    tooltip: '"Open user folder"'
```

Note that `template`, `tooltip` and `onClick` function bodies are defined as code. This code will be evaluated at runtime using the [mathjs](https://mathjs.org/) evaluate function, similar to how Conditional Layouts work.

Also, observe that to write literal strings, you must use double quotes.

```yaml
 tooltip: '"Open user folder"'
```

## Evaluation Scope

When we say "each type has its own evaluation scope," we refer to how variables and functions within each type are accessible and interact during runtime.

In the context of the Toolbar Widget documentation, each type (such as generic or text, date, and power) has its own set of variables that it can access and manipulate. These variables and functions are defined within the scope of each type, meaning they are accessible and meaningful only within that particular type.

Therefore, by stating that "each type has its own evaluation scope," we emphasize that the variables and functions defined within each type are isolated and tailored to the specific functionality and requirements of that type within the Toolbar Widget.

| Type | Scope |
| ---- | ----- |
| `generic` or `text` | `icon`, `window`, `env` |
| `date` | `icon`, `window`, `env`, `date` |
| `power` | `icon`, `window`, `env`, `power` |

### Generic Module
**Scope:** `icon` is a object of all available icons in [react-icons](https://react-icons.github.io/react-icons/), `env` is the current environment, and `window` is the current focused window.


### Date Module
**Scope:** The `date` variable represents the current date and time as a string, formatted according to the specified format in the `format` property of the Module.

**Format:** is a string to parse the date in a specific format, follow the next guide of [moment-js](https://momentjscom.readthedocs.io/en/latest/moment/04-displaying/01-format/)

### Power Module
**Scope:** The `power` variable has available the same properties exposed in the [win32 SYSTEM_POWER_STATUS interface](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status)

## Icons in Templates
### `icon.Name` vs `"[ICON:Name:12]"`

When it comes to icons, you have two options: using the variable `icon.Name` or the string `"[ICON:Name:12]"`. 

It's important to note that `icon.Name` will be internally converted to `"[ICON:Name]"`. You can use whichever you prefer; the only difference is that with the quoted version, you can specify the size of the icon in pixels.
