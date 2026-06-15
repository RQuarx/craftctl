# Architecture

This document describes the internal structure of the Minecraft Java launcher.

The project is organized by responsibility. Each module should own one clear part of the launcher.

## Source layout

```text
src/
├── main.rs
├── app.rs
├── error.rs
├── result.rs
├── config.rs
│
├── tui/
│   ├── mod.rs
│   ├── terminal.rs
│   ├── events.rs
│   ├── state.rs
│   ├── theme.rs
│   ├── screens/
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   ├── instances.rs
│   │   ├── mods.rs
│   │   ├── downloads.rs
│   │   ├── settings.rs
│   │   └── logs.rs
│   └── widgets/
│       ├── mod.rs
│       ├── navbar.rs
│       ├── progress.rs
│       ├── modal.rs
│       └── table.rs
│
├── minecraft/
│   ├── mod.rs
│   ├── version.rs
│   ├── manifest.rs
│   ├── assets.rs
│   ├── libraries.rs
│   ├── arguments.rs
│   ├── classpath.rs
│   └── launch.rs
│
├── auth/
│   ├── mod.rs
│   ├── microsoft.rs
│   ├── minecraft.rs
│   ├── tokens.rs
│   └── keyring.rs
│
├── instances/
│   ├── mod.rs
│   ├── instance.rs
│   ├── profile.rs
│   ├── manager.rs
│   └── storage.rs
│
├── mods/
│   ├── mod.rs
│   ├── modrinth.rs
│   ├── curseforge.rs
│   ├── github.rs
│   ├── resolver.rs
│   └── installed.rs
│
├── loaders/
│   ├── mod.rs
│   ├── fabric.rs
│   ├── forge.rs
│   ├── quilt.rs
│   └── neoforge.rs
│
├── download/
│   ├── mod.rs
│   ├── client.rs
│   ├── task.rs
│   ├── queue.rs
│   ├── checksum.rs
│   └── progress.rs
│
├── java/
│   ├── mod.rs
│   ├── detect.rs
│   ├── version.rs
│   └── runtime.rs
│
├── storage/
│   ├── mod.rs
│   ├── paths.rs
│   ├── cache.rs
│   ├── json.rs
│   └── fs.rs
│
└── util/
    ├── mod.rs
    ├── platform.rs
    ├── process.rs
    └── time.rs
```

## Module responsibilities

### `main.rs`

Application entry point.

It should only initialize the program and call `app::run()`.

### `app.rs`

Coordinates the whole application.

Responsibilities:

```text
- load config
- initialize logging
- initialize terminal
- create application state
- start the main event loop
- cleanly shut down the app
```

Keep `app.rs` thin. If startup wiring grows beyond a few local bindings,
move dependency construction into `app/` submodules such as `app/context.rs`.
Those submodules are still part of the application coordination layer.

Avoid passing a broad app context into lower layers unless they truly need it.
Prefer building the context in `app`, then passing each module only the specific
dependencies it uses.

### `error.rs`

Defines the main launcher error type.

Use this for structured errors such as:

```text
- IO errors
- network errors
- JSON parsing errors
- authentication errors
- download errors
- launch errors
```

### `result.rs`

Defines the shared result alias.

Example:

```rust
pub type Result<T> = std::result::Result<T, crate::error::LauncherError>;
```

### `config.rs`

Handles launcher configuration.

Examples:

```text
- default Minecraft directory
- selected Java runtime
- UI settings
- download settings
- memory settings
```

## `tui/`

Terminal user interface.

Responsibilities:

```text
- drawing screens
- handling keyboard input
- managing focused widgets
- displaying progress
- showing errors and logs
```

Suggested files:

```text
terminal.rs     terminal setup and cleanup
events.rs       keyboard and terminal events
state.rs        TUI-specific state
theme.rs        UI colors and styles
screens/        full application screens
widgets/        reusable UI components
```

## `minecraft/`

Vanilla Minecraft installation and launch logic.

Responsibilities:

```text
- fetch version manifests
- parse Minecraft version metadata
- download client jars
- download libraries
- download assets
- build JVM arguments
- build game arguments
- build classpath
- launch Minecraft
```

Suggested files:

```text
version.rs      Minecraft version types
manifest.rs     Mojang version manifest handling
assets.rs       asset index and asset downloads
libraries.rs    library resolution
arguments.rs    JVM and game arguments
classpath.rs    classpath generation
launch.rs       process spawning
```

## `auth/`

Microsoft and Minecraft authentication.

Responsibilities:

```text
- Microsoft OAuth login
- Minecraft services authentication
- token refresh
- token storage
- account selection
```

Suggested files:

```text
microsoft.rs    Microsoft OAuth flow
minecraft.rs    Minecraft services API
tokens.rs       token data structures
keyring.rs      secure token storage
```

## `instances/`

Minecraft instances and profiles.

An instance is a separate Minecraft installation/profile.

Example instances:

```text
- Vanilla 1.21
- Fabric 1.20.1
- Forge modpack
- Modrinth pack
```

Responsibilities:

```text
- create instances
- delete instances
- edit instance settings
- store instance metadata
- resolve instance game directory
```

Suggested files:

```text
instance.rs     instance data structure
profile.rs      launch profile settings
manager.rs      create/load/update/delete instances
storage.rs      read/write instance files
```

## `mods/`

Mod platform integration and installed mod management.

Responsibilities:

```text
- search mods
- download mods
- update mods
- read installed mods
- resolve dependencies
- detect incompatible mods
```

Suggested files:

```text
modrinth.rs     Modrinth API support
curseforge.rs   CurseForge API support
github.rs       GitHub Releases support
resolver.rs     dependency/version resolution
installed.rs    installed mod scanning
```

## `loaders/`

Mod loader support.

Responsibilities:

```text
- install mod loaders
- resolve loader versions
- patch launch arguments when needed
- provide loader-specific metadata
```

Suggested files:

```text
fabric.rs       Fabric support
forge.rs        Forge support
quilt.rs        Quilt support
neoforge.rs     NeoForge support
```

## `download/`

Download system.

Responsibilities:

```text
- HTTP downloads
- download queue
- progress tracking
- retries
- checksum validation
- temporary files
```

Suggested files:

```text
client.rs       shared HTTP client
task.rs         single download task
queue.rs        multiple download tasks
checksum.rs     SHA-1/SHA-256 verification
progress.rs     progress state
```

## `java/`

Java detection and runtime management.

Responsibilities:

```text
- find installed Java versions
- validate Java version compatibility
- select runtime per instance
- use bundled/runtime-downloaded Java if supported
```

Suggested files:

```text
detect.rs       search system for Java
version.rs      parse Java version output
runtime.rs      selected Java runtime model
```

## `storage/`

Filesystem, paths, cache, and persistence.

Responsibilities:

```text
- app directories
- config directory
- cache directory
- data directory
- JSON read/write helpers
- filesystem helpers
```

Suggested files:

```text
paths.rs        OS-specific launcher paths
cache.rs        cache handling
json.rs         JSON read/write helpers
fs.rs           filesystem utilities
```

## `util/`

Small shared utilities.

Responsibilities:

```text
- platform helpers
- process helpers
- time helpers
- small reusable functions
```

Avoid putting large business logic here.

## Runtime data layout

The launcher should store runtime data using OS-specific directories.

On Linux, the layout may look like this:

```text
~/.local/share/your-launcher/
├── instances/
│   ├── vanilla-1.21/
│   │   ├── instance.toml
│   │   ├── minecraft/
│   │   ├── mods/
│   │   ├── resourcepacks/
│   │   └── saves/
│   └── fabric-1.20.1/
│       ├── instance.toml
│       ├── minecraft/
│       └── mods/
│
├── versions/
├── libraries/
├── assets/
└── logs/
```

Config files:

```text
~/.config/your-launcher/
└── config.toml
```

Cache files:

```text
~/.cache/your-launcher/
├── downloads/
├── manifests/
├── modrinth/
└── curseforge/
```

Use the `directories` crate to resolve these paths instead of hardcoding them.

## Instance layout

Each instance should be isolated.

Example:

```text
instances/fabric-1.20.1/
├── instance.toml
├── minecraft/
│   ├── options.txt
│   ├── servers.dat
│   ├── saves/
│   ├── resourcepacks/
│   └── shaderpacks/
├── mods/
└── logs/
```

Example `instance.toml`:

```toml
name = "Fabric 1.20.1"
minecraft_version = "1.20.1"
loader = "fabric"
loader_version = "latest"
java_runtime = "system"
memory_min_mb = 1024
memory_max_mb = 4096
```

## Dependency direction

Modules should depend inward.

Recommended direction:

```text
tui
 ↓
app
 ↓
instances ── minecraft ── download
 ↓              ↓
storage       java
 ↓
util
```

Avoid making low-level modules depend on the TUI.

Bad:

```text
download -> tui
minecraft -> tui
auth -> tui
```

Good:

```text
tui -> download progress state
tui -> minecraft launch status
app -> coordinates modules
```

## Error handling

Use structured errors in `error.rs`.

Example:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid Minecraft version: {0}")]
    InvalidMinecraftVersion(String),

    #[error("Java runtime not found")]
    JavaNotFound,

    #[error("launch failed: {0}")]
    LaunchFailed(String),
}
```

## Naming rules

Use clear module names.

Good:

```text
resolver.rs
manifest.rs
classpath.rs
arguments.rs
installed.rs
```

Avoid vague names:

```text
stuff.rs
helper.rs
manager2.rs
misc.rs
```

## Testing layout

Use `tests/` for integration tests.

```text
tests/
├── manifest_tests.rs
├── instance_tests.rs
├── download_tests.rs
└── java_tests.rs
```

Use unit tests inside modules for small internal behavior.

Example:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn parses_version() {
        // test code
    }
}
```

## Future workspace layout

When the project becomes large, split it into a Cargo workspace.

```text
minecraft-launcher/
├── Cargo.toml
├── crates/
│   ├── launcher-core/
│   ├── launcher-tui/
│   ├── launcher-auth/
│   └── launcher-cli/
└── docs/
```

Suggested crates:

```text
launcher-core     Minecraft install, launch, instances, mods
launcher-tui      Ratatui interface
launcher-auth     Microsoft/Minecraft authentication
launcher-cli      final binary
```

Start as a single crate first. Split only when the code becomes hard to manage.
