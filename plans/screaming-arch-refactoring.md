# Screaming Architecture Refactoring Plan

## Goals

1. Организовать код по бизнес-фичам, не по техническим слоям
2. Инвертировать зависимости (DIP): domain/business-logic не зависит от infrastructure
3. Разделить API handlers и use cases (application services)
4. Устранить God Objects (AppServices, FullRepository)

## Target Structure

```
src/
├── main.rs
├── error.rs
│
├── shared_infra/
│   └── sqlite_db.rs              # Arc<SqlitePool> wrapper (NEW)
│
├── supporters/                    # Фича supporters
│   ├── module.rs                 # pub mod api; pub mod domain; pub mod infra;
│   ├── domain.rs                 # SupporterRepository trait, models, InitData
│   ├── api.rs                    # HTTP handlers
│   └── infra.rs                  # SQLite implementation
│
├── push/                         # Фича push-уведомлений
│   ├── module.rs
│   ├── domain.rs                 # PushSubscriptionRepository trait
│   ├── api.rs
│   ├── infra.rs                  # SQLite implementation (includes PushSubscriptionRow conversion)
│   └── client.rs                 # PushClient (web-push logic)
│
├── oauth/                        # Фича OAuth авторизации
│   ├── module.rs
│   ├── domain.rs                 # OAuthService trait + TWITCH_SCOPES
│   ├── api.rs                    # OAuth handlers
│   └── infra.rs                  # TwitchOAuthProvider + TWITCH_SCOPES_VALIDATOR
│
├── token_manager/                # Фича управления токенами
│   ├── module.rs
│   ├── domain.rs                 # TokenProvider, TokenRepository traits
│   ├── application.rs            # TokenManagerS (use case)
│   └── infra/
│       └── sqlite.rs             # SqliteTokenRepository (includes TokenRecordRow conversion)
│
├── eventsub/                     # Фича Twitch EventSub
│   ├── module.rs
│   ├── domain.rs                 # StreamLifecycle, ChatHandler traits
│   ├── application.rs            # TwitchLifecycle, EventSubManager
│   ├── api.rs                    # Eventsub HTTP receiver (if any)
│   └── infra/
│       ├── client.rs             # TwitchApiClient + TwitchApiClientTrait
│       └── listener.rs           # EventSubClient + start_eventsub_task
│
├── auth/                         # Фича админской авторизации
│   ├── module.rs
│   ├── domain.rs                 # AdminSessionRepository, AdminWhiteListRepository traits
│   ├── api.rs                    # Admin auth handlers
│   └── infra.rs                  # SQLite implementations
│
├── state/                        # In-memory state management
│   ├── module.rs
│   ├── domain.rs                 # StateRepository trait
│   ├── application.rs            # AppState + StateManager
│   └── infra/
│       └── in_memory.rs          # InMemoryStateRepository
│
├── app/                         # DI Composition Root
│   ├── mod.rs                   # App struct + AppBuilder
│   └── ports.rs                  # All service traits
│
└── router.rs                     # Axum router assembly
```

---

## Phase 1: Create Feature Modules (Scaffolding)

- [ ] **1.1** Create `src/supporters/module.rs` with:
  ```rust
  pub mod api;
  pub mod domain;
  pub mod infra;
  ```
- [ ] **1.2** Create `src/push/module.rs` with:
  ```rust
  pub mod api;
  pub mod domain;
  pub mod infra;
  pub mod client;
  ```
- [ ] **1.3** Create `src/oauth/module.rs` with:
  ```rust
  pub mod api;
  pub mod domain;
  pub mod infra;
  ```
- [ ] **1.4** Create `src/token_manager/module.rs` with:
  ```rust
  pub mod domain;
  pub mod application;
  pub mod infra;
  ```
- [ ] **1.5** Create `src/eventsub/module.rs` with:
  ```rust
  pub mod domain;
  pub mod application;
  pub mod infra;
  ```
- [ ] **1.6** Create `src/auth/module.rs` with:
  ```rust
  pub mod api;
  pub mod domain;
  pub mod infra;
  ```
- [ ] **1.7** Create `src/state/module.rs` with:
  ```rust
  pub mod domain;
  pub mod application;
  pub mod infra;
  ```

**Verification:**
- [ ] `cargo check` passes

---

## Phase 2: Move Domain Layer (Traits + Models)

### 2.1 supporters/domain.rs

**From:** `src/supporters/repository.rs`, `src/supporters.rs`
**Contents:**
- `SupporterRepositoryData` struct
- `SupporterRepository` trait (all methods)

- [ ] Create `src/supporters/domain.rs`
- [ ] Move `SupporterRepositoryData`
- [ ] Move `SupporterRepository` trait

### 2.2 push/domain.rs

**From:** `src/push/repository.rs`
**Contents:**
- `PushSubscription` struct
- `PushSubscriptionRepository` trait

- [ ] Create `src/push/domain.rs`
- [ ] Move `PushSubscription`
- [ ] Move `PushSubscriptionRepository`

### 2.3 oauth/domain.rs

**From:** `src/providers/twitch/auth.rs` (TWITCH_SCOPES), new
**Contents:**
- `OAuthService` trait
- `TWITCH_SCOPES` constant
- `TWITCH_SCOPES_VALIDATOR` constant

- [ ] Create `src/oauth/domain.rs`
- [ ] Move `TWITCH_SCOPES` from `providers/twitch/auth.rs`
- [ ] Move `TWITCH_SCOPES_VALIDATOR`
- [ ] Create `OAuthService` trait stub

### 2.4 token_manager/domain.rs

**From:** `src/providers/token_repository.rs`, `src/token_manager/token_provider.rs` (MOVE, not copy)
**Contents:**
- `ProviderVariant` enum
- `AccountVariant` enum
- `TokenRecord` struct
- `TokenEnum` enum
- `TokenRepository` trait
- `TokenProvider` trait

- [ ] Create `src/token_manager/domain.rs`
- [ ] Move `ProviderVariant` from `providers/token_repository.rs`
- [ ] Move `AccountVariant`
- [ ] Move `TokenRecord`
- [ ] Move `TokenEnum`
- [ ] Move `TokenRepository` trait
- [ ] Move `TokenProvider` trait from `token_manager/token_provider.rs`

### 2.5 eventsub/domain.rs

**From:** `src/app_logic.rs`
**Contents:**
- `StreamLifecycle` trait
- `ChatHandler` trait

- [ ] Create `src/eventsub/domain.rs`
- [ ] Move `StreamLifecycle` trait
- [ ] Move `ChatHandler` trait

### 2.6 auth/domain.rs

**From:** `src/auth_service/admin_session_repository.rs`, `src/auth_service/admin_whitelist_repository.rs`
**Contents:**
- `AdminSessionInfo` struct
- `AdminSessionRepository` trait
- `AdminWhiteListRepository` trait

- [ ] Create `src/auth/domain.rs`
- [ ] Move `AdminSessionInfo`
- [ ] Move `AdminSessionRepository`
- [ ] Move `AdminWhiteListRepository`

### 2.7 state/domain.rs

**From:** `src/app_state.rs` (partially)
**Contents:**
- `StateRepository` trait

- [ ] Create `src/state/domain.rs`
- [ ] Create `StateRepository` trait

**Phase 2 Verification:**
- [ ] `cargo check` passes
- [ ] No infra imports in domain modules

---

## Phase 3: Create Shared Infrastructure

### 3.1 shared_infra/sqlite_db.rs

**Purpose:** All features share one `SqlitePool` via `SqliteDb`. Keep as reusable wrapper.

**From:** `src/infrastructure/db_sqlite.rs` (refactor)
**Contents:** `SqliteDb` struct wrapping `SqlitePool`

- [ ] Create `src/shared_infra.rs`
- [ ] Create `src/shared_infra/sqlite_db.rs`
- [ ] Move `SqliteDb` struct (or recreate)
- [ ] Add `pool()` method returning `&SqlitePool`

**Phase 3 Verification:**
- [ ] `cargo check` passes

---

## Phase 4: Create Infrastructure Implementations

### 4.1 supporters/infra.rs

**From:** `src/infrastructure/db_sqlite/supporters.rs`
**Contents:**
- `SqliteSupporterRepository` struct
- Implementation of `SupporterRepository`

- [ ] Create `src/supporters/infra.rs`
- [ ] Create `SqliteSupporterRepository` struct
- [ ] Implement `SupporterRepository`

### 4.2 push/infra.rs

**From:** `src/infrastructure/db_sqlite/push_subscriptions.rs`, `src/infrastructure/db_sqlite/models.rs` (PushSubscriptionRow)
**Contents:**
- `SqlitePushSubscriptionRepository` struct
- `PushSubscriptionRow` → `PushSubscription` conversion
- Implementation of `PushSubscriptionRepository`

- [ ] Create `src/push/infra.rs`
- [ ] Create `SqlitePushSubscriptionRepository` struct
- [ ] Move `PushSubscriptionRow` from `models.rs`
- [ ] Move `From<PushSubscriptionRow> for PushSubscription`
- [ ] Implement `PushSubscriptionRepository`

### 4.3 oauth/infra.rs

**From:** `src/providers/twitch/token_provider.rs`
**Contents:**
- `TwitchOAuthProvider` struct implementing `OAuthService`
- `TWITCH_SCOPES_VALIDATOR` (if not in domain)

- [ ] Create `src/oauth/infra.rs`
- [ ] Move `TwitchTokenProvider` from `providers/twitch/token_provider.rs`
- [ ] Rename to `TwitchOAuthProvider`
- [ ] Implement `OAuthService`
- [ ] Move `TWITCH_SCOPES_VALIDATOR` if needed

### 4.4 token_manager/infra/sqlite.rs

**From:** `src/infrastructure/db_sqlite/tokens.rs`, `src/infrastructure/db_sqlite/models.rs` (TokenRecordRow)
**Contents:**
- `SqliteTokenRepository` struct
- `TokenRecordRow` → `TokenRecord` conversion
- Implementation of `TokenRepository`

- [ ] Create `src/token_manager/infra/mod.rs`
- [ ] Create `src/token_manager/infra/sqlite.rs`
- [ ] Move `TokenRecordRow` from `models.rs`
- [ ] Move `TryFrom<TokenRecordRow> for TokenRecord`
- [ ] Create `SqliteTokenRepository` struct
- [ ] Implement `TokenRepository`

### 4.5 eventsub/infra/client.rs

**From:** `src/providers/twitch/client.rs`
**Contents:**
- `TwitchApiClient` struct
- `TwitchApiClientTrait` trait (new abstraction)

- [ ] Create `src/eventsub/infra/mod.rs`
- [ ] Create `src/eventsub/infra/client.rs`
- [ ] Create `TwitchApiClientTrait` trait
- [ ] Move `TwitchApiClient` struct
- [ ] Make it implement `TwitchApiClientTrait`

### 4.6 eventsub/infra/listener.rs

**From:** `src/providers/twitch/eventsub.rs`
**Contents:**
- `EventSubClient` struct
- `start_eventsub_task()` function
- `create_eventsub_shutdown_channel()` function

- [ ] Create `src/eventsub/infra/listener.rs`
- [ ] Move `EventSubClient`
- [ ] Move `start_eventsub_task`
- [ ] Move `create_eventsub_shutdown_channel`

### 4.7 auth/infra.rs

**From:** `src/infrastructure/db_sqlite/admin_session.rs`, `src/infrastructure/db_sqlite/admin_whitelist.rs`
**Contents:**
- `SqliteAdminSessionRepository` struct
- `SqliteAdminWhiteListRepository` struct
- Implementation of both traits

- [ ] Create `src/auth/infra.rs`
- [ ] Move `SqliteAdminSessionRepository` from `admin_session.rs`
- [ ] Move `SqliteAdminWhiteListRepository` from `admin_whitelist.rs`
- [ ] Implement traits

### 4.8 state/infra/in_memory.rs

**From:** `src/app_state.rs`
**Contents:**
- `InMemoryStateRepository` struct

- [ ] Create `src/state/infra/mod.rs`
- [ ] Create `src/state/infra/in_memory.rs`
- [ ] Create `InMemoryStateRepository`
- [ ] Implement `StateRepository`

**Phase 4 Verification:**
- [ ] `cargo check` passes
- [ ] All infra modules have no domain dependencies on each other

---

## Phase 5: Create Application Services

### 5.1 token_manager/application.rs

**From:** `src/token_manager.rs`
**Contents:**
- `TokenManagerS` struct
- All methods with `Arc<dyn TokenRepository>` (not FullRepository)

- [ ] Create `src/token_manager/application.rs`
- [ ] Create `TokenManagerS` struct
- [ ] Change field from `Arc<dyn FullRepository>` to `Arc<dyn TokenRepository>`
- [ ] Move all methods from `token_manager.rs`

### 5.2 eventsub/application.rs

**From:** `src/app_services.rs`
**Contents:**
- `TwitchLifecycle` struct
- `TwitchStreamLifecycleAdapter` struct
- `TwitchChatHandlerAdapter` struct
- `EventSubManager` struct

- [ ] Create `src/eventsub/application.rs`
- [ ] Move `TwitchStreamLifecycleAdapter`
- [ ] Move `TwitchChatHandlerAdapter`
- [ ] Move `TwitchLifecycle`
- [ ] Move `EventSubManager`
- [ ] Change `EventSubManager` to use `Arc<dyn TwitchApiClientTrait>`

### 5.3 state/application.rs

**From:** `src/app_state.rs`
**Contents:**
- `AppState` struct (rename to avoid conflict)
- `StateManager` struct
- `create_state()` function

- [ ] Create `src/state/application.rs`
- [ ] Create `AppState` (rename to avoid conflict with axum Extension)
- [ ] Create `StateManager`
- [ ] Move `create_state()`

**Phase 5 Verification:**
- [ ] `cargo check` passes
- [ ] Application services depend only on domain traits

---

## Phase 6: Create App Composition Root

### 6.1 app/ports.rs

**Purpose:** Define all service traits that App exposes to API handlers

**Contents:**
- `SupportersService` trait
- `PushService` trait
- `OAuthService` trait
- `TokenService` trait
- `StateRepository` trait (if exposed to handlers)

- [ ] Create `src/app/ports.rs`
- [ ] Define `SupportersService`
- [ ] Define `PushService`
- [ ] Define `OAuthService`
- [ ] Define `TokenService`

### 6.2 app/mod.rs

**From:** `src/app_services.rs`, `src/app_state.rs`
**Contents:**
- `App` struct (DI container)
- `AppBuilder` (builder pattern)

- [ ] Create `src/app/mod.rs`
- [ ] Create `App` struct with all service arcs
- [ ] Create `AppBuilder`
- [ ] Implement builder methods for each service
- [ ] Move `PushClient` initialization here (DI for PushClient)

**Phase 6 Verification:**
- [ ] `cargo check` passes
- [ ] No circular dependencies

---

## Phase 7: Migrate API Handlers

### 7.1 supporters/api.rs

**From:** `src/api/supporters.rs`
**Changes:** Use `SupportersService` trait instead of `AppServices`

- [ ] Create `src/supporters/api.rs`
- [ ] Move all handlers from `api/supporters.rs`
- [ ] Change `Extension<AppServices>` to `Extension<Arc<dyn SupportersService>>`
- [ ] Change handlers to use service trait methods

### 7.2 push/api.rs

**From:** `src/api/push_subscriptions.rs`
**Changes:** Use `PushService` trait

- [ ] Create `src/push/api.rs`
- [ ] Move all handlers
- [ ] Change extensions to use `PushService`
- [ ] `PushClient` injected via `App`

### 7.3 oauth/api.rs

**From:** `src/api/auth.rs`
**Changes:** Use `OAuthService` trait

- [ ] Create `src/oauth/api.rs`
- [ ] Move all handlers
- [ ] Change extensions to use `OAuthService`

### 7.4 auth/api.rs

**Purpose:** Admin auth handlers
**From:** New or future split from admin handlers

- [ ] Create `src/auth/api.rs`
- [ ] Define admin auth handlers

**Phase 7 Verification:**
- [ ] `cargo check` passes
- [ ] API handlers have no direct infra imports

---

## Phase 8: Router Assembly

### 8.1 router.rs

**From:** `src/api.rs`
**Changes:** Import from feature modules, use `App`

- [ ] Rename `src/api.rs` to `src/router.rs`
- [ ] Update imports to use feature `api.rs` modules
- [ ] Change `AppState` + `AppServices` to single `App`
- [ ] Update route registrations
- [ ] Add OpenAPI documentation

**Phase 8 Verification:**
- [ ] `cargo check` passes
- [ ] All routes work

---

## Phase 9: Update Main.rs

### 9.1 main.rs

**Changes:** Use new `AppBuilder` pattern

- [ ] Update module declarations
- [ ] Replace `create_state` + `AppServices::builder` with `AppBuilder`
- [ ] Remove old module imports
- [ ] Verify startup

**Phase 9 Verification:**
- [ ] `cargo check` passes
- [ ] Application starts successfully

---

## Phase 10: Remove Old Files

**Files to remove after full migration:**

- [ ] `src/api.rs` (replaced by router.rs)
- [ ] `src/api/` directory
- [ ] `src/app_services.rs`
- [ ] `src/app_state.rs`
- [ ] `src/app_logic.rs`
- [ ] `src/infrastructure.rs`
- [ ] `src/infrastructure/` directory
- [ ] `src/providers.rs`
- [ ] `src/providers/` directory
- [ ] `src/auth_service.rs`
- [ ] `src/auth_service/` directory
- [ ] `src/supporters.rs`
- [ ] `src/supporters/` directory
- [ ] `src/push.rs`
- [ ] `src/push/` directory
- [ ] `src/token_manager.rs`
- [ ] `src/token_manager/` (partially - keep module structure)

**Phase 10 Verification:**
- [ ] `cargo check` passes
- [ ] All old code removed
- [ ] All tests pass
- [ ] `just gen` generates valid OpenAPI

---

## Dependency Flow After Refactoring

```
API Handlers → Service Traits (app/ports.rs) → App (DI Root)
     │                                        ▲
     │                                        │
     └──────▶ SupportersService ───▶ SupporterRepository ───▶ SqliteSupporterRepository
     └──────▶ PushService ────────▶ PushSubscriptionRepository ───▶ SqlitePushSubscriptionRepository
     └──────▶ OAuthService ────────▶ TwitchOAuthProvider
     └──────▶ TokenService ────────▶ TokenRepository ───▶ SqliteTokenRepository
     └──────▶ StateRepository ─────▶ InMemoryStateRepository
     └──────▶ TwitchApiClientTrait ─▶ TwitchApiClient
```

---

## Key Refactoring Decisions

### Decision 1: Traits in Domain, Implementations in Infra
Each feature has `domain.rs` with traits and `infra.rs` with implementations.

### Decision 2: No FullRepository Composite
Each feature's application service depends only on its specific repository trait.

### Decision 3: StateRepository Trait for Caching
`AppState` becomes `StateRepository` trait with in-memory implementation.

### Decision 4: PushClient as Infrastructure
`PushClient::from_env()` replaced with proper dependency injection via `AppBuilder`.

### Decision 5: Feature Modules Own Their API
Each feature owns its HTTP handlers in `feature/api.rs`.

### Decision 6: App is the DI Container
`App` struct contains all service traits. `AppBuilder` constructs everything.

### Decision 7: Shared SqliteDb
All infra modules share one `SqliteDb` wrapper around `SqlitePool`.

---

## Progress Tracking

| Phase | Status |
|-------|--------|
| Phase 1: Scaffolding | [ ] |
| Phase 2: Domain | [ ] |
| Phase 3: Shared Infra | [ ] |
| Phase 4: Infrastructure | [ ] |
| Phase 5: Application Services | [ ] |
| Phase 6: App Composition | [ ] |
| Phase 7: API Handlers | [ ] |
| Phase 8: Router | [ ] |
| Phase 9: Main.rs | [ ] |
| Phase 10: Cleanup | [ ] |

---

## Verification Checklist (After Each Phase)

- [ ] `cargo check` passes
- [ ] No import cycles
- [ ] No direct infrastructure imports in API handlers
- [ ] All tests pass (`cargo test`)
- [ ] OpenAPI generation works (`just gen`)
