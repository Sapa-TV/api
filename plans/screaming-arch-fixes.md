---

## Issue 6: PushClient Direct Instantiation (DI Violation) 🔄 IN PROGRESS

**Problem**: `test_push_all` handler in `push/api.rs` creates `PushClient::from_env()` directly instead of using injected dependency.

**Solution**:
- Add `PushClient` to App via AppBuilder
- Change `test_push_all` to use `app.push_client`

**Files to change**:
- [ ] `src/app/app.rs` - add `push_client` field
- [ ] `src/main.rs` - create and inject PushClient
- [ ] `src/push/api.rs` - use injected client

---

## Issue 7: State Module as Cache Layer 🔄 IN PROGRESS

**Problem**: State module exists but is not integrated. It duplicates SupporterRepository. User wants it to be a cache layer for API.

**Solution**:
```
New flow:
API → CachingSupportersService → InMemoryStateRepository (cache)
                         ↓ (cache miss)
                  SqliteSupporterRepository → SQLite
```

**Architecture**:
1. `CachedSupportersService` wraps `InMemoryStateRepository` (cache) + `SqliteSupporterRepository` (persistence)
2. Read: check cache first, fallback to SQLite
3. Write: update SQLite, then update cache (cache invalidation)
4. Load cache from DB at startup

**Files to change**:
- [ ] `src/app/app.rs` - add `state: Arc<InMemoryStateRepository>` to App
- [ ] `src/app/service_adapters.rs` - create `CachedSupportersService` decorator
- [ ] `src/main.rs` - wire up state cache

---

## Verification Checklist

- [ ] `cargo check` passes
- [ ] All tests pass
- [ ] PushClient injected via AppBuilder
- [ ] State module used as cache layer
- [ ] Cache invalidation works on writes