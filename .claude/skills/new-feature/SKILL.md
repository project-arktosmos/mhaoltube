# Skill: New Full-Stack Feature

Scaffold a new feature across the stack (backend, frontend).

## Instructions

1. Ask the user for:
   - Feature/entity name (e.g. "playlists", "bookmarks")
   - What API endpoints are needed
   - Whether it needs new database tables (defined in Rust backend)

2. **Backend**:
   - Create `packages/tauri/src-tauri/src/api/{feature}.rs`
   - Implement `pub fn router() -> Router<AppState>`
   - Add `pub mod {feature};` to `src/api/mod.rs`
   - Register in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
   - If new database access is needed, add repo to `AppState` in `src/lib.rs`
   - Run checks: `cargo check -p mhaoltube-desktop && cargo test -p mhaoltube-desktop`

3. **Frontend**:
   - Define types in `packages/frontend/src/types/{feature}.type.ts`
   - Create adapter in `packages/frontend/src/adapters/classes/{feature}.adapter.ts`
   - Create service in `packages/frontend/src/services/{feature}.service.ts`
   - Create component directory `packages/frontend/src/components/{feature}/`
   - Follow all component rules: no `<style>` tags, no inline styles, use `classnames`, TypeScript `$props()` with inline types, callback props for parent communication

4. **Verify**:
   - Run `pnpm lint && pnpm check && pnpm build && pnpm test`
   - Fix any errors before committing

5. **Commit** each logical step separately (backend, frontend) with short English messages.
