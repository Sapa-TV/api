# Screaming Architecture Fixes

## Issue 1: Duplicate OAuthService trait ✅ DONE

**Problem**: `OAuthService` defined in both `oauth/domain.rs:6` and `app/ports.rs:27`

**Fix**:
- [x] Remove `OAuthService` from `app/ports.rs`
- [x] Keep trait in `oauth/domain.rs` via `pub use`
- [x] Re-export from `app/ports.rs` via `pub use crate::oauth::domain::OAuthService;`
- [x] Export `TwitchOAuthService` from `service_adapters.rs` via `pub use`
- [x] All tests pass, cargo check passes

---

## Issue 2: App (DI container) incomplete

**Problem**: App only has supporters, push, oauth. The following are missing:
- `token_manager`
- `eventsub`
- `auth`
- `state`

**Fix**:
- Add missing service traits to `app/ports.rs`
- Add missing services to `App` struct
- Add missing services to `AppBuilder`
- Update `main.rs` to inject all services through AppBuilder

---

## Issue 3: Missing TokenService in ports.rs

**Problem**: TokenManager has domain traits but they're not in ports.rs, breaking abstraction. Other services can't depend on token management through an interface.

**Fix**:
- Create `TokenService` trait in `app/ports.rs`
- Define required methods (refresh_token, get_valid_token, etc.)
- Update `SqliteTokenRepository` to implement TokenService trait
- Update application services to depend on `dyn TokenService`

---

## Issue 4: Inconsistent module structure

**Problem**: Features have different structures:
- `oauth`, `supporters`, `push`: no `application.rs`
- `token_manager`, `eventsub`, `state`: have `application.rs`

**Fix**:
- If `application.rs` is needed for business logic orchestration → add to all features
- If it was optional → remove from features that don't need it
- Document which features need application layer

---

## Issue 5: Direct infrastructure usage in main.rs

**Problem**: `main.rs` directly imports `token_manager::infra::sqlite::SqliteTokenRepository`

**Fix**:
- Inject through AppBuilder like other services
- Hide implementation details behind trait

---

## Dependency Flow (Target)

```
API Handlers → Service Traits (app/ports.rs) → App (DI Root)
     │                                        ▲
     │                                        │
     ├──▶ SupportersService ──▶ SupporterRepository
     ├──▶ PushService ────────▶ PushSubscriptionRepository
     ├──▶ OAuthService ───────▶ TwitchOAuthProvider
     ├──▶ TokenService ───────▶ TokenRepository
     ├──▶ AuthService ────────▶ TwitchAuthProvider
     ├──▶ EventSubService ────▶ EventSubListener
     └──▶ StateService ───────▶ InMemoryStateRepository
```

---

## Verification Checklist

- [x] `cargo check` passes
- [x] No duplicate trait definitions (OAuthService fixed)
- [ ] App contains all services (token_manager, eventsub, auth, state still missing)
- [ ] All services injected through AppBuilder
- [ ] No direct infrastructure imports in API handlers
- [x] All tests pass
- [x] OpenAPI generation works