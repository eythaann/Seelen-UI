set windows-shell := ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

default:
    @just --list

# build the background and run the app in dev mode (UI served with hot reload)
dev:
    cargo build
    npx tauri dev

# format everything (TS/JSON/Markdown/CSS with dprint, Rust with rustfmt)
fmt:
    npx dprint fmt
    cargo fmt

# verify formatting without writing changes
fmt-check:
    npx dprint check
    cargo fmt -- --check

# lint JS/TS/Svelte (oxlint) and Rust (clippy)
lint:
    npx oxlint
    cargo clippy --all-targets -- -D warnings

# type check the frontend, the core lib and the Rust code
check:
    npm run type-check
    cargo check

# run all tests (JS + Rust)
test:
    npm test
    cargo test

# translate the i18n files (run only right before the final commit)
translate:
    npx tsx ./scripts/translate/mod.ts

# start the development of a new release (bumps versions and commits)
version-start version:
    npx tsx ./scripts/versionish.ts start {{version}}

# finish the current release (updates changelog, commits and tags)
version-finish:
    npx tsx ./scripts/versionish.ts finish

# update the tauri dependencies (npm + cargo)
tauri-update:
    npx tsx ./scripts/UpdateTauri.ts
