# CLI Contract — `focus config` Subcommand

## Synopsis

```
focus config set <key> <value>
focus config get <key>
```

## Keys

| Key | Values | Description |
|---|---|---|
| `theme` | `dark`, `light`, `material`, `onedark` | UI color theme override; `auto` resets to OS detection |
| `vim-mode` | `true`, `false` | Enable/disable hjkl + dd navigation |

## Examples

```bash
# Set theme
focus config set theme material
# → Saved: theme = material

# Set vim mode
focus config set vim-mode true
# → Saved: vim-mode = true

# Get current theme
focus config get theme
# → theme = material

# Reset theme to OS auto-detect
focus config set theme auto
# → Saved: theme = auto (auto-detect)
```

## Error Cases

```
focus config set color blue
# → Error: unknown key 'color'. Valid keys: theme, vim-mode

focus config set theme neon
# → Error: unknown theme 'neon'. Valid themes: dark, light, material, onedark, auto

focus config set vim-mode yes
# → Error: invalid value 'yes' for vim-mode. Use 'true' or 'false'
```

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Invalid key or value |
| 2 | Config file write failed (printed to stderr) |
