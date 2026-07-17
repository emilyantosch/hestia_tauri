# Filesystem Synchronization and Watcher Refactor Plan

## Decision

Hestia will use a **reconciliation-based synchronization model**.

- The filesystem is the source of truth.
- The database is a materialized index of filesystem state.
- Watcher events are low-latency change hints, not an authoritative event log.
- A reconciler is the only component responsible for bringing the database toward filesystem state.

This preserves a responsive experience without assuming that filesystem notifications are complete, ordered, portable, or durable.

The responsibilities can be summarized as:

```text
Reconciler = correctness
Watcher    = acceleration
Hashes     = content evidence
```

The reconciler must remain usable without a watcher. In that mode, startup, manual, or periodic root reconciliation still converges correctly, but changes are discovered with greater latency and traversal cost.

## Why Retain the Watcher?

Hashes do not reveal changes without inspecting the filesystem. If no watcher or filesystem journal identifies likely changes, every synchronization cycle must:

1. traverse every directory;
2. compare every indexed path and object type;
3. read metadata for every object;
4. potentially read file contents to recompute hashes.

A stored folder or root hash does not avoid this work for changes made outside Hestia: Hestia must still traverse the tree to calculate the new hash. Directory modification times are also insufficient because changing a deeply nested file does not reliably update every ancestor directory.

For a root containing `N` objects where `k` objects changed:

- periodic root reconciliation is approximately `O(N)` in directory entries per cycle;
- full content rehashing is approximately `O(total file bytes)` per cycle;
- watcher-directed reconciliation is approximately `O(k)` plus enumeration of affected directories during normal operation;
- recovery remains `O(N)`, but happens only at startup or when watcher state is uncertain.

The watcher is therefore worthwhile for large libraries or low-latency updates. For small libraries with relaxed refresh requirements, periodic reconciliation is a valid simpler operating mode. This should remain a configuration/deployment choice rather than a correctness dependency.

## Goals

1. Reflect ordinary filesystem changes quickly without repeatedly scanning every watched root.
2. Recover automatically from missed events, watcher overflow, application downtime, transient I/O errors, and races.
3. Ensure watcher and scanner paths use exactly the same indexing policy.
4. Make synchronization observable and testable.
5. Prevent a partially failed scan from deleting valid database records.
6. Keep UI and thumbnail notifications tied to committed database changes.

## Non-goals

- Strong consistency with arbitrary external filesystem writers.
- Treating `notify` events as a durable event stream.
- Eliminating all startup enumeration. Avoiding startup reconciliation would require platform-specific filesystem journals and persistent checkpoints.
- Preserving the current watcher event types as the service boundary.

## Consistency Contract

The index is **eventually consistent** with the filesystem.

Under normal operation, affected paths should converge shortly after the watcher debounce window. A watched root is considered healthy only when:

- its watcher was installed successfully;
- its latest reconciliation completed successfully;
- no watcher overflow or unresolved synchronization error is pending.

Errors must leave a path or root marked dirty. They must not be logged and forgotten.

## Proposed Architecture

```text
notify backend
    |
    v
WatchSource -----> ChangeHint
                       |
                       v
                SyncCoordinator
                 - coalescing
                 - dirty scopes
                 - retries
                 - health state
                       |
                       v
                   Reconciler
                 - inspect current state
                 - calculate changes
                 - transactional writes
                       |
                       v
                   Repository
                       |
                       v
              committed ChangeSet
                 /             \
                UI          thumbnails
```

### `WatchSource`

A thin adapter over `notify`/`notify-debouncer-full` that:

- installs and removes recursive watches;
- converts backend events into hints containing the affected root and paths;
- reports backend errors and overflow explicitly;
- does not hash filesystem objects;
- does not classify an object from `Path::is_dir()` after it may have disappeared;
- does not write to the database.

Suggested hint model:

```rust
struct ChangeHint {
    root: CanonPath,
    paths: Vec<PathBuf>,
    reason: ChangeReason,
}

enum ChangeReason {
    Changed,
    Renamed,
    WatcherError,
    Overflow,
}
```

Event kinds should only influence reconciliation scope. They must not determine the database operation directly.

Hints form a transient dirty set, not a durable ledger. If Hestia exits before processing them, startup root reconciliation repairs the index. If a durable audit or work ledger is later needed, it should be written from committed `ChangeSet`s rather than raw watcher events.

### `SyncCoordinator`

The coordinator owns synchronization state for each watched root:

- `Starting`
- `Reconciling`
- `Healthy`
- `Dirty`
- `Failed`
- `Stopped`

It will:

- coalesce duplicate and related paths;
- promote a set of child paths to their common parent when appropriate;
- serialize reconciliation for a root;
- retain dirty state while work is in progress;
- retry transient failures with bounded backoff;
- promote watcher errors, overflow, and queue saturation to root reconciliation;
- provide acknowledged `watch`, `unwatch`, `status`, and `shutdown` commands.

Ordering of watcher events does not need to be trusted because every job inspects current filesystem state.

Use bounded work queues. If a queue cannot accept more detailed hints, mark the whole root dirty rather than spawning unbounded tasks or silently dropping state.

### `Reconciler`

The reconciler compares current filesystem state with the database and emits a `ChangeSet`. It should expose operations at multiple scopes:

- `reconcile_path(root, path)` for an ordinary file change or deletion;
- `reconcile_directory(root, path)` for direct-child changes;
- `reconcile_subtree(root, path)` for a new or moved directory tree;
- `reconcile_root(root)` for startup and recovery.

Expected behavior:

- Existing file: stat/hash as required, then insert or update by path.
- Missing path: remove its exact database record and, if it represented a directory, all indexed descendants.
- Existing directory: compare its children against indexed children.
- Rename: reconcile both old and new paths in one logical unit.
- Directory moved into a root: reconcile the subtree because backends may not emit events for every existing descendant.
- Directory rename/delete: update or remove descendant paths, not only the directory row.

Database changes for one logical reconciliation should be transactional. A `ChangeSet` is published only after commit.

### `Repository`

Repository APIs should accept filesystem snapshots or calculated mutations rather than watcher-specific event objects. The repository should not depend on `notify::EventKind`.

Add database invariants where appropriate:

- unique indexed path within `files`;
- unique indexed path within `folders`;
- indexes supporting exact-path and subtree queries.

Use path as the primary projection key. Filesystem identifiers can assist rename correlation, but must account for hard links before being made unique.

## Startup Sequence

The current scan-then-watch order has a race. Replace it with:

1. Validate and canonicalize configured roots.
2. Install recursive watches and begin buffering/coalescing hints.
3. Reconcile each root.
4. Drain the dirty hints accumulated during reconciliation.
5. Mark the root healthy after the dirty set reaches quiescence.
6. Generate thumbnails and notify the UI from committed `ChangeSet`s.

Cached database content may be shown immediately while startup reconciliation runs in the background. “Ready” and “index verified” can be represented separately if useful to the UI.

## Normal Operation

1. Receive watcher hints.
2. Coalesce them for a short window, initially 100–300 ms and configurable.
3. Select the smallest safe reconciliation scope.
4. Inspect current filesystem state.
5. Commit database changes.
6. Emit one aggregated `ChangeSet` for UI and thumbnail work.

A two-second debounce is likely unnecessarily visible for ordinary edits. Correctness should come from reconciliation and coalescing, not a long debounce.

## Recovery Strategy

Perform root reconciliation:

- on every application/library startup;
- after watcher overflow or backend errors;
- after a watched root disappears and later returns;
- after queue saturation;
- after suspend/resume if the platform exposes it;
- optionally during idle time as a low-frequency safety check.

A startup root scan can be made cheaper by enumerating path, type, size, modification time, and filesystem identifier first. Hash file contents only when metadata or stored state indicates a possible change. Platform filesystem journals can be considered later if startup enumeration becomes a measured bottleneck.

## Shared Indexing Policy

Create one policy used by both root and targeted reconciliation:

- ignored directory names;
- ignored extensions;
- symlink behavior;
- maximum file size;
- permission/error behavior;
- canonicalization and root-boundary checks;
- supported object types.

Currently the watcher indexes objects that the scanner may ignore. That guarantees disagreement and must be removed.

A partial or unreadable directory must be represented as an incomplete scan. Records below an incomplete scope must not be deleted merely because they were absent from scan output.

## File Hash Strategy

Hashes verify content; they do not decide whether a path exists, was deleted, or moved. Reconciliation should keep these concepts separate:

```text
path                  Current location and primary projection key
filesystem_id         Rename-correlation hint
size + modified_at    Cheap change-detection metadata
content_hash          Evidence for the current file bytes
```

The current `identity_hash` combines content, filesystem identity, and filename. It changes when the file is edited or renamed, so it is not a stable identity and should not be used as one. Prefer explicit fields with clear semantics. Filesystem identifiers also require care around hard links and identifier reuse.

For a hinted file, the reconciler should:

1. read path, type, size, modification time, and filesystem identifier;
2. compare that cheap metadata with the database;
3. skip content hashing when the stored state is demonstrably current;
4. hash a new file or a file whose relevant metadata changed;
5. update metadata without content-triggered downstream work when the content hash is unchanged;
6. delete a missing path without attempting to hash it.

Metadata comparison is an optimization, not an absolute proof that bytes are unchanged on every filesystem. Root reconciliation may apply stricter rules where required.

Hashing must account for concurrent writes:

1. read metadata before hashing;
2. stream the file through BLAKE3 rather than loading the entire file into memory;
3. read metadata again;
4. commit the hash only if size, modification time, and filesystem identity still match;
5. otherwise leave the path dirty and retry.

If the file disappears during hashing, reconcile it as missing. Do not discard the error and assume the previous database state remains valid.

If inline hashing makes large new files visibly slow, split processing into:

1. structural reconciliation that promptly stores the path and metadata with a `Pending` hash state;
2. bounded background content hashing that later commits the hash if the stable-read checks pass.

Content hashes should drive content-sensitive work such as thumbnail regeneration and duplicate detection. Metadata-only changes should not trigger that work. Hash representation must be standardized, preferably as raw bytes or canonical lowercase hexadecimal rather than mixed debug/display strings.

## Folder Hash Decision

`FolderHash::hash()` recursively hashes the whole subtree. Running it for a single watcher event is expensive, and changing a file currently leaves stored ancestor folder hashes stale.

Before implementing the reconciler, decide whether aggregate folder hashes are required.

Preferred options, in order:

1. Remove aggregate folder content hashes if no feature requires them.
2. Compute them lazily when requested.
3. Maintain them incrementally and mark ancestors dirty after child changes.

Do not recursively hash an entire large subtree on every ordinary event.

## Existing Correctness Issues to Fix First

### Watcher

In `crates/services/src/fs/watcher.rs`:

- `init_watcher()` reports success even when debouncer creation fails.
- backend and event-processing errors are logged and discarded;
- `unwatch()` does not call the underlying watcher's `unwatch()`;
- `WatchPath` and `UnwatchPath` commands have no acknowledgement;
- `GetWatchPaths` may fail to answer when no paths exist;
- an unwatch error can terminate the watcher loop;
- event hashing races with subsequent rename/delete operations;
- directory classification depends partly on `path.is_dir()` after the event;
- direct event-to-database writes do not reconcile directory descendants;
- bounded channel pressure is converted into spawned tasks rather than explicit dirty state.

In `crates/model/src/services/mod.rs`, replace the infallible `From<PathBuf> for CanonPath`. A canonicalization failure currently becomes an empty path. Implement `TryFrom<PathBuf>`/`TryFrom<&Path>` and return the actual error.

### Scanner

In `crates/services/src/fs/scanner.rs` and `crates/repositories/src/fs/operations.rs`:

- `get_directory_state()` returns file state only but is reused for folders;
- folder reconciliation can generate `DeleteFile` operations for all database files;
- the final folder deletion call passes `delete_file_batch` instead of `delete_folder_batch`;
- batch file insert/update counters are reversed;
- file and folder state need distinct metadata/query APIs;
- prefix-based `LIKE "root%"` queries can include sibling paths with the same prefix;
- per-file processing failures can omit a file and later cause an unsafe deletion;
- reported batch errors still allow synchronization to appear complete;
- root scanning and concurrent filesystem changes are not coordinated with the watcher.

Also standardize hash serialization. Scanner models use hash display strings while event upserts use debug formatting, which can make equal hashes compare as different values.

## Implementation Phases

### Phase 1: Establish a Correct Reconciliation Baseline

- Fix the scanner defects listed above.
- Separate file and folder database snapshots.
- Track scan completeness per directory.
- Centralize indexing policy.
- Standardize path and hash representations.
- Store the cheap comparison metadata needed by reconciliation, including size and modification time.
- Separate stable filesystem identity, content hash, and path semantics instead of relying on the current combined identity hash.
- Add path uniqueness constraints and supporting indexes after cleaning existing duplicates.
- Make root reconciliation idempotent.

Exit criteria:

- Running root reconciliation twice produces no second change set.
- Reconciliation never deletes records from unreadable or incomplete scopes.
- Database state matches a deterministic filesystem fixture.

### Phase 2: Extract the Reconciler

- Move comparison and mutation logic out of watcher event handling.
- Add path, directory, subtree, and root reconciliation APIs.
- Introduce transactional `ChangeSet` results.
- Remove repository dependencies on watcher event types.
- Implement metadata-first file comparison and stable-read content hashing.
- Ensure `ChangeSet`s distinguish metadata changes from content changes.
- Decide and implement folder hash behavior.

Exit criteria:

- Tests can reconcile changes without constructing `notify` events.
- Create, modify, delete, file rename, directory rename, and moved-in trees converge correctly.

### Phase 3: Replace the Watcher with a Hint Source

- Implement `WatchSource` with acknowledged commands.
- Add explicit watcher error and overflow signals.
- Call the backend when unwatching.
- Remove hashing and database access from the watcher.
- Replace silent canonicalization fallback with errors.

Exit criteria:

- Watch management failures are returned to the caller.
- Backend errors always mark the affected root dirty.
- Watcher tests verify hints and lifecycle rather than exact database mutations.

### Phase 4: Add the Coordinator

- Add per-root dirty sets and health state.
- Coalesce hints and choose reconciliation scope.
- Add bounded retries and root-level fallback.
- Implement watch-before-reconcile startup sequencing.
- Emit committed changes to UI and thumbnail consumers.

Exit criteria:

- Changes made during startup reconciliation are not lost.
- Injected overflow results in root reconciliation.
- Queue saturation cannot cause permanent divergence.

### Phase 5: Performance and Operational Hardening

- Measure startup enumeration, hashing, reconciliation latency, and queue depth.
- Measure and optimize metadata-first change detection.
- Tune debounce and batch sizes from measurements.
- Add optional idle safety reconciliation.
- Evaluate deferred/background hashing for large files if inline hashing is a measured latency problem.
- Evaluate filesystem journals only if root enumeration is a demonstrated problem.
- Measure whether small-library deployments benefit from a watcher or can use periodic reconciliation alone.

## Testing Strategy

### Unit tests

- dirty-path coalescing and scope promotion;
- root-boundary and canonical-path validation;
- indexing policy decisions;
- file/folder snapshot comparison;
- incomplete-directory deletion protection;
- rename calculation;
- coordinator state transitions and retry behavior.

### Integration tests

For every scenario, wait for convergence and compare the complete database projection with filesystem state:

- create, modify, and delete a file;
- rapid create/modify/delete before debounce expires;
- rename a file;
- create, rename, move in, and delete a directory tree;
- modify files while root reconciliation is running;
- make changes while the application is stopped, then restart;
- ignored files and directories;
- inaccessible directories and transient permission failures;
- watcher overflow/error injection;
- database write failure followed by retry;
- watched root removal and reappearance;
- duplicate and overlapping watch requests;
- clean unwatch and shutdown.

Avoid tests that require one exact platform-specific `notify::EventKind`. Assert eventual database convergence instead.

### Property/state-machine tests

Generate sequences of filesystem operations, apply them to a temporary tree, run reconciliation, and assert:

- every indexable filesystem object has exactly one database record;
- no database record refers to a missing object;
- ignored objects have no records;
- parent relationships are valid;
- a second reconciliation is a no-op.

## Observability

Record structured events and metrics for:

- watched root health and state transitions;
- watcher errors and overflow;
- dirty path count and scope promotions;
- reconciliation duration and scope;
- files/folders inserted, updated, deleted, and skipped;
- incomplete directories and retry counts;
- queue saturation;
- time from watcher hint to committed change.

Errors should include the root, affected scope, operation, and whether reconciliation remains pending.

## Completion Criteria

The refactor is complete when:

- no watcher event directly mutates the database;
- every watcher failure has a reconciliation path;
- startup has no scan/watch gap;
- ordinary changes use targeted reconciliation;
- full root reconciliation repairs changes made while Hestia was not running;
- reconciliation is idempotent and safe under partial I/O failure;
- watcher behavior is tested by convergence rather than platform event shape;
- content hashes are computed only when needed and are committed from a stable read;
- metadata-only changes do not trigger content-sensitive downstream work;
- synchronization still functions through root reconciliation when the watcher is disabled or unavailable;
- the UI and thumbnail pipeline react only to committed change sets.
