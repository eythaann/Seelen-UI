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

---

## ❌ Modules Requiring Migration (Legacy Pattern)

### 4. **media** ❌

- **File**: `src/background/modules/media/infrastructure.rs`
- **Current Pattern**:
  - Uses `register_media_events()` function called manually
  - Direct `emit_to_webviews` calls from event subscribers
  - No lazy event registration
- **Migration Priority**: HIGH (core functionality)
- **Estimated Effort**: Medium
- **Notes**: Complex module with multiple media sources (players, devices, sessions)

### 5. **notifications** ❌

- **File**: `src/background/modules/notifications/infrastructure.rs`
- **Current Pattern**:
  - Uses `register_notification_events()` function called manually
  - Uses `trace_lock!` macro instead of proper singleton
  - Has `release_notification_events()` cleanup function
- **Migration Priority**: HIGH (user-facing)
- **Estimated Effort**: Medium
- **Notes**: Needs careful handling of COM activation callbacks

### 6. **network** ❌

- **File**: `src/background/modules/network/infrastructure.rs`
- **Current Pattern**:
  - Uses `register_network_events()` with `AtomicBool` for registration tracking
  - Direct event emission in subscription
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Low-Medium
- **Notes**: Relatively straightforward migration

### 7. **apps** ❌

- **File**: `src/background/modules/apps/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Unknown

### 8. **power** ❌

- **File**: `src/background/modules/power/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Unknown

### 9. **user** ❌

- **File**: `src/background/modules/user/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: MEDIUM
- **Estimated Effort**: Unknown

### 10. **language** ❌

- **File**: `src/background/modules/language/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: LOW (less frequently used)
- **Estimated Effort**: Unknown

### 11. **input** ❌

- **File**: `src/background/modules/input/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: LOW (specialized functionality)
- **Estimated Effort**: Unknown

### 12. **system_settings** ❌

- **File**: `src/background/modules/system_settings/infrastructure.rs`
- **Current Pattern**: Unknown (needs analysis)
- **Migration Priority**: LOW
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
