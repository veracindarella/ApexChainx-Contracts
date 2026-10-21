# Governance Module — NatSpec / Doc-Comment Reference

> **Note:** Soroban is a Rust-based smart contract platform. The equivalent of
> Solidity's `@notice` / `@param` / `@return` NatSpec annotations are Rust
> `///` doc comments. This document provides the full NatSpec-style reference
> for every public governance function. The authoritative source of truth is
> the inline `///` comments in
> [`apexchainx_calculator/src/governance.rs`](../apexchainx_calculator/src/governance.rs).
>
> Resolves issue #820.

---

## Overview

The Governance module implements two orthogonal role-management flows:

| Flow | Functions | Auth required |
|------|-----------|---------------|
| Admin two-step transfer | `propose_admin` → `accept_admin` | Current admin proposes; proposed admin accepts |
| Admin cancellation | `cancel_admin_proposal` | Current admin only |
| Admin renounce (irreversible) | `renounce_admin` | Current admin only |
| Operator two-step transfer | `propose_operator` → `accept_operator` | Admin proposes; proposed operator accepts |
| Operator cancellation | `cancel_operator_proposal` | Current admin only |
| Operator single-step (break-glass) | `set_operator` | Admin only; no new-operator consent |
| Read-only queries | `get_pending_admin`, `get_pending_operator` | None |

---

## Constants

### `PROPOSAL_EXPIRY_WINDOW`

```
pub(crate) const PROPOSAL_EXPIRY_WINDOW: u64 = 90 * 24 * 60 * 60;
```

**@notice** Duration in ledger-seconds after which a pending admin or operator
proposal automatically expires. Currently set to 90 days.

**@dev** Both the admin and operator handoff paths share this single constant.
Boundary tests (`± 1 second`) pin the exact expiry behaviour.

---

## Functions

### `propose_admin`

```rust
pub fn propose_admin(env: &Env, caller: &Address, new_admin: &Address) -> Result<(), SLAError>
```

**@notice** Initiates a two-step admin transfer by recording `new_admin` as the
pending admin. The transfer is not complete until `new_admin` calls
`accept_admin`.

**@dev**
- Requires current admin authorization (`require_admin`).
- Blocked when the contract config is frozen (`require_not_frozen`).
- Storage version must match (`check_version`).
- `new_admin` must differ from both the current admin and the caller (`require_new_admin_distinct`).
- If a prior pending admin exists it is **superseded**: an `adm_sup` event is
  emitted first carrying `(previous_candidate, new_admin)`, then an `adm_prop`
  event carrying `(new_admin,)`.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Address of the current admin initiating the proposal.  
**@param** `new_admin` — Address of the proposed new admin.

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::Unauthorized)` if `caller` is not the current admin.  
**@return** `Err(SLAError::InvalidInput)` if `new_admin` equals the current admin or `caller`.  
**@return** `Err(SLAError::ConfigFrozen)` if the config is frozen.

**@emits** `adm_sup (v, caller) → (previous, new_admin)` — only when superseding.  
**@emits** `adm_prop (v, caller) → (new_admin,)`

---

### `accept_admin`

```rust
pub fn accept_admin(env: &Env, caller: &Address) -> Result<(), SLAError>
```

**@notice** Completes the two-step admin transfer. Must be called by the address
that was proposed via `propose_admin`.

**@dev**
- Calls `require_auth()` on `caller`, requiring an explicit signature from the
  proposed admin.
- Verifies `caller` matches the stored pending admin.
- Checks the proposal has not expired (`require_proposal_valid`).
- On success: overwrites `ADMIN_KEY`, removes `PENDING_ADMIN_KEY` and
  `PENDING_ADMIN_TS_KEY`, emits `adm_acc`.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the pending admin address.

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::NoPendingTransfer)` if no proposal exists.  
**@return** `Err(SLAError::ProposalExpired)` if the proposal window has lapsed.  
**@return** `Err(SLAError::Unauthorized)` if `caller` is not the pending admin.

**@emits** `adm_acc (v, caller) → ()`

---

### `cancel_admin_proposal`

```rust
pub fn cancel_admin_proposal(env: &Env, caller: &Address) -> Result<(), SLAError>
```

**@notice** Cancels an active pending admin proposal. Only the current admin may
cancel.

**@dev**
- Returns `NoPendingTransfer` if no proposal is stored.
- Removes `PENDING_ADMIN_KEY` and `PENDING_ADMIN_TS_KEY`.
- Emits `adm_can`.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the current admin.

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::NoPendingTransfer)` if no active proposal exists.  
**@return** `Err(SLAError::Unauthorized)` if `caller` is not the current admin.

**@emits** `adm_can (v, caller) → ()`

---

### `get_pending_admin`

```rust
pub fn get_pending_admin(env: &Env) -> Result<Option<Address>, SLAError>
```

**@notice** Returns the currently proposed admin address, or `None` if no
proposal is active.

**@dev** Read-only. Does not require authorization. Storage version must match.

**@param** `env` — Soroban execution environment.

**@return** `Ok(Some(address))` when a proposal is pending.  
**@return** `Ok(None)` when no proposal is active.

---

### `renounce_admin`

```rust
pub fn renounce_admin(env: &Env, caller: &Address) -> Result<(), SLAError>
```

**@notice** Permanently removes the admin role. **Irreversible.** After this
call no admin-gated function can ever be called again on this contract instance.

**@dev**
- Sets `ADMIN_RENOUNCED_KEY = true`.
- Removes `ADMIN_KEY`, `PENDING_ADMIN_KEY`, `PENDING_ADMIN_TS_KEY`.
- If a pending operator proposal exists it is also cleared and an `op_can`
  event is emitted first (an adminless contract must not allow stale operator
  handoffs to fire).
- Emits `adm_ren` last.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the current admin.

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::Unauthorized)` if `caller` is not the current admin.

**@emits** `op_can (v, caller) → ()` — only if a pending operator proposal existed.  
**@emits** `adm_ren (v, caller) → ()`

---

### `propose_operator`

```rust
pub fn propose_operator(env: &Env, caller: &Address, new_operator: &Address) -> Result<(), SLAError>
```

**@notice** Initiates a two-step operator transfer. The new operator must call
`accept_operator` to complete the handoff.

**@dev**
- Requires admin authorization.
- Blocked when config is frozen.
- Does **not** restrict `new_operator` to be distinct from the current operator
  (unlike `propose_admin`).
- Supersession semantics: if a prior pending operator exists, an `op_sup` event
  is emitted before the `op_prop` event.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the current admin.  
**@param** `new_operator` — Proposed new operator address.

**@return** `Ok(())` on success.

**@emits** `op_sup (v, caller) → (previous, new_operator)` — only when superseding.  
**@emits** `op_prop (v, caller) → (new_operator,)`

---

### `accept_operator`

```rust
pub fn accept_operator(env: &Env, caller: &Address) -> Result<(), SLAError>
```

**@notice** Completes the two-step operator handoff. Must be called by the
proposed operator; requires their explicit authorization.

**@dev**
- Calls `require_auth()` on `caller`.
- Validates proposal has not expired.
- Overwrites `OPERATOR_KEY`, clears pending slots, emits `op_acc`.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the pending operator address.

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::NoPendingTransfer)` if no proposal exists.  
**@return** `Err(SLAError::ProposalExpired)` if the window has lapsed.  
**@return** `Err(SLAError::Unauthorized)` if `caller` is not the pending operator.

**@emits** `op_acc (v, caller) → ()`

---

### `cancel_operator_proposal`

```rust
pub fn cancel_operator_proposal(env: &Env, caller: &Address) -> Result<(), SLAError>
```

**@notice** Cancels an active pending operator proposal. Only the current admin
may cancel.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the current admin.

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::NoPendingTransfer)` if no active operator proposal exists.

**@emits** `op_can (v, caller) → ()`

---

### `get_pending_operator`

```rust
pub fn get_pending_operator(env: &Env) -> Result<Option<Address>, SLAError>
```

**@notice** Returns the currently proposed operator address, or `None` if no
proposal is active.

**@return** `Ok(Some(address))` when a proposal is pending.  
**@return** `Ok(None)` when no proposal is active.

---

### `set_operator`

```rust
pub fn set_operator(env: &Env, caller: &Address, new_operator: &Address) -> Result<(), SLAError>
```

**@notice** Directly replaces the operator address (single-step, admin only).
Use as a **break-glass** mechanism only; prefer the two-step
`propose_operator`/`accept_operator` path for routine rotations because it
requires the new operator's explicit consent.

**@dev**
- Does **not** call `require_auth()` on `new_operator`.
- If a pending two-step operator proposal exists, it is cancelled first (`op_can`
  event emitted) so a stale handoff can never override this direct assignment.
  (#569)
- Pending *admin* proposals are unaffected.
- Requires admin role, config not frozen, version match.

**@param** `env` — Soroban execution environment.  
**@param** `caller` — Must be the current admin.  
**@param** `new_operator` — New operator address (no consent required).

**@return** `Ok(())` on success.  
**@return** `Err(SLAError::Unauthorized)` if `caller` is not the current admin.

**@emits** `op_can (v, caller) → ()` — only if a pending operator proposal existed.  
**@emits** `op_set (v, caller) → (new_operator,)`

---

## Error Reference

| Error | Meaning |
|-------|---------|
| `SLAError::Unauthorized` | Caller does not hold the required role |
| `SLAError::NoPendingTransfer` | No pending proposal exists for the requested operation |
| `SLAError::ProposalExpired` | The proposal was created more than `PROPOSAL_EXPIRY_WINDOW` seconds ago |
| `SLAError::InvalidInput` | Input violates an invariant (e.g. proposed admin same as current admin) |
| `SLAError::ConfigFrozen` | Contract config is frozen; governance mutations blocked |
| `SLAError::NotInitialized` | Contract has not been initialized |

---

## Event Reference

| Event symbol | Emitter function | Payload |
|---|---|---|
| `adm_prop` | `propose_admin` | `(new_admin,)` |
| `adm_sup` | `propose_admin` | `(previous_candidate, new_admin)` |
| `adm_acc` | `accept_admin` | `()` |
| `adm_can` | `cancel_admin_proposal` | `()` |
| `adm_ren` | `renounce_admin` | `()` |
| `adm_xp` | `require_proposal_valid` | `(stale_admin,)` |
| `op_prop` | `propose_operator` | `(new_operator,)` |
| `op_sup` | `propose_operator` | `(previous_op, new_operator)` |
| `op_acc` | `accept_operator` | `()` |
| `op_can` | `cancel_operator_proposal` / `set_operator` / `renounce_admin` | `()` |
| `op_set` | `set_operator` | `(new_operator,)` |
| `op_xp` | `require_proposal_valid` | `(stale_operator,)` |

All events are published as `(symbol, version, caller) → payload`.
