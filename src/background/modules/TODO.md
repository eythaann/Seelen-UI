# System Modules Architecture Migration TODO

This document tracks the migration of system modules from legacy architecture patterns to the modern architecture
pattern.

## Modern Architecture Pattern

The modern pattern uses:

- **Singleton lazy initialization** with `LazyLock`
- **Lazy event registration** with `Once` in infrastructure/handlers file
- **Separation of concerns** between system logic (application.rs) and Tauri integration (infrastructure.rs)
- **Event-driven design** using `event_manager!` macro for internal events
- **Thread-safe** implementations

See `CLAUDE.md` section "System Modules Architecture (Modern Pattern)" for complete documentation and examples.

---

## ✅ Modules Using Modern Pattern (Complete)

### 1. **monitors** ✅

- **File**: `src/background/modules/monitors/infrastructure.rs`
- **Pattern**: Uses `Once` for lazy event registration
- **Status**: ✅ MODERN - Reference implementation

### 2. **radios** ✅

- **File**: `src/background/modules/radios/handlers.rs`
- **Pattern**: Uses `Once` for lazy event registration, proper separation of concerns
- **Status**: ✅ MODERN - Reference implementation

### 3. **start** ✅

- **File**: `src/background/modules/start/infrastructure.rs`
- **Pattern**: Recently migrated to modern pattern
- **Status**: ✅ MODERN - Reference implementation

### 4. **media** ✅

- **Files**: `src/background/modules/media/{devices,players}/infrastructure.rs`
- **Pattern**: Split into two separate managers (DevicesManager and PlayersManager)
- **Status**: ✅ MODERN - Reference implementation
- **Notes**:
  - Split into separate `devices` and `players` modules
  - Both follow modern pattern with LazyLock and Once
  - Drop trait implemented for automatic resource cleanup

---

### 5. **notifications** ✅

- **File**: `src/background/modules/notifications/infrastructure.rs`
- **Pattern**: Uses `Once` for lazy event registration, LazyLock singleton
- **Status**: ✅ MODERN - Reference implementation
- **Notes**:
  - WinRT event handler stored only as i64 token
  - Drop trait implemented for automatic cleanup
  - LOADED_NOTIFICATIONS uses Mutex<HashSet> for thread-safe notification tracking

### 6. **power** ✅

- **File**: `src/background/modules/power/infrastructure.rs`
- **Pattern**: Uses `Once` for lazy event registration, LazyLock singleton with `instance()` method
- **Status**: ✅ MODERN - Reference implementation
- **Notes**:
  - Implements wake-up event handling with 2-second timeout to refresh stale power state
  - Uses `Arc<Mutex<>>` for thread-safe mutable state access from Windows callbacks
  - Proper separation between system logic (application.rs) and Tauri integration (infrastructure.rs)

### 7. **system_settings** ✅

- **File**: `src/background/modules/system_settings/infrastructure.rs`
- **Pattern**: Uses `Once` for lazy event registration, LazyLock singleton with `instance()` method
- **Status**: ✅ MODERN - Reference implementation
- **Notes**:
  - Migrated from `lazy_static!` to `LazyLock`
  - Only stores i64 tokens, not TypedEventHandler instances
  - Proper separation between system logic (application.rs) and Tauri integration (infrastructure.rs)
  - Removed unnecessary thread spawn from event registration

---

## ❌ Modules Requiring Migration (Legacy Pattern)

### 8. **network** ❌

- **File**: `src/background/modules/network/infrastructure.rs`
- **Current Pattern**:
  - Uses `register_network_events()` with `AtomicBool` for registration tracking
  - Direct event emission in subscription
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Low-Medium
- **Notes**: Relatively straightforward migration

### 9. **apps** ❌

- **File**: `src/background/modules/apps/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Unknown

### 10. **user** ❌

- **File**: `src/background/modules/user/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Unknown

### 11. **language** ❌

- **File**: `src/background/modules/language/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: LOW (less frequently used)
- **Estimated Effort**: Unknown

### 12. **input** ❌

- **File**: `src/background/modules/input/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: LOW (specialized functionality)
- **Estimated Effort**: Unknown

### 13. **system_tray** ❌

- **File**: `src/background/modules/system_tray/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: HIGH (core UI component)
- **Estimated Effort**: Unknown

---

## Migration Checklist

When migrating a module to the modern pattern, ensure:

### Infrastructure/Handlers Layer

- [ ] Create private `get_<module>_manager()` function
- [ ] Use `static TAURI_EVENT_REGISTRATION: Once` for lazy event registration
- [ ] Subscribe to manager events inside `Once::call_once`
- [ ] Emit to webviews only from event subscription (not from business logic)
- [ ] All Tauri commands call the private getter function
- [ ] No direct `emit_to_webviews` calls from business logic

### Application Layer

- [ ] Implement singleton pattern with `LazyLock`
- [ ] Define event enum for internal events
- [ ] Use `event_manager!` macro for subscription system
- [ ] Separate `new()` and `init()` functions
- [ ] Move system event listeners to `setup_listeners()`
- [ ] Remove Tauri-specific code from application layer
- [ ] Use `ResultLogExt::log_error()` for error handling

### WinRT Event Handler Best Practices

**IMPORTANT: TypedEventHandlers do NOT need to be stored**

- Windows-rs clones handlers internally, so storing them is unnecessary
- Only store event tokens (as `i64` for WinRT, `EventRegistrationToken` for Win32)
- Create handlers inline when registering events

**Wrapper Structs for Automatic Resource Management**

- For WinRT objects with event subscriptions, create wrapper structs
- Register events in `create()` or `new()` method, store tokens
- Unregister events in `Drop` implementation
- Use wrappers instead of mirror structs to encapsulate COM lifecycle

**Example Pattern:**

```rust
pub struct WinRTObjectWrapper {
    pub object: SomeWinRTObject,
    event_token: i64,  // WinRT uses i64
}

impl WinRTObjectWrapper {
    pub fn create(object: SomeWinRTObject) -> Result<Self> {
        let token = object.SomeEvent(&TypedEventHandler::new(Self::on_event))?;
        Ok(Self { object, event_token: token })
    }
}

impl Drop for WinRTObjectWrapper {
    fn drop(&mut self) {
        self.object.RemoveSomeEvent(self.event_token).log_error();
    }
}
```

### Commands & Events (libs/core)

- [ ] Add getter command in `libs/core/src/handlers/commands.rs`
- [ ] Add changed event in `libs/core/src/handlers/events.rs`
- [ ] Regenerate TypeScript bindings: `cd libs/core && deno task build:rs`

### Integration

- [ ] Add import in `src/background/exposed.rs`
- [ ] Export event type from module's `mod.rs`
- [ ] Remove any manual event registration calls
- [ ] Verify compilation with `cargo check`
- [ ] Test lazy initialization behavior

---

## Benefits of Migration

1. **Consistent Architecture**: All modules follow the same pattern, easier to understand and maintain
2. **Better Performance**: Lazy initialization reduces startup time
3. **Separation of Concerns**: Clear boundary between system logic and Tauri integration
4. **Thread Safety**: Proper use of synchronization primitives
5. **Testability**: System logic can be tested independently from Tauri
6. **Event-Driven**: Decoupled event system allows multiple subscribers

---

## Notes

- Migration should be done incrementally, one module at a time
- Test thoroughly after each migration
- Legacy patterns will continue to work but should be updated when touching the code
- Reference implementations: `monitors`, `radios`, `start`
