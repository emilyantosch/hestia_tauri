# Thumbnail Pipeline Implementation Plan

## Current Status (Completed)

### ✅ Phase 1: Core Infrastructure (COMPLETED)
- **Dependencies Added**: `image` crate (v0.25) and `infer` crate (v0.16) 
- **Type-Safe Enums**: `ThumbnailSize` with compile-time dimensions and string conversions
- **Core Generator**: `ThumbnailGenerator` struct with configurable filter types
- **Image Processing**: Aspect ratio preservation using `image::thumbnail()`
- **File Icon Generation**: Color-coded backgrounds for non-image file types  
- **Error Handling**: Custom `ThumbnailError` enum with proper context using `thiserror`
- **Thumbnail Struct**: Rich domain model with SeaORM integration methods
- **SeaORM Integration**: Direct conversion to/from `ActiveModel` and `Model` entities

**Files Modified:**
- `/src-tauri/Cargo.toml` - Added dependencies
- `/src-tauri/src/file_system/thumbnails.rs` - Complete implementation

## Remaining Work

### 📋 Phase 2: Database Operations
**Goal**: Implement repository pattern for thumbnail persistence with batch processing

#### Step 7: ThumbnailRepository Implementation
- [ ] Create `ThumbnailRepository` struct in `/src-tauri/src/database/`
- [ ] Implement CRUD operations:
  - `create_thumbnail(file_id, thumbnail) -> Result<thumbnails::Model>`
  - `get_by_file_and_size(file_id, size) -> Result<Option<Thumbnail>>`
  - `get_thumbnails_for_file(file_id) -> Result<Vec<Thumbnail>>`
  - `delete_thumbnails_for_file(file_id) -> Result<u64>`
  - `get_thumbnail_by_id(id) -> Result<Option<Thumbnail>>`

#### Step 8: Batch Processing Queries  
- [ ] Implement efficient batch operations:
  - `get_files_without_thumbnails(size, limit) -> Result<Vec<i32>>`
  - `batch_create_thumbnails(Vec<(file_id, Thumbnail)>) -> Result<u64>`
  - `get_thumbnail_stats() -> Result<ThumbnailStats>` (counts by size/type)
- [ ] Add database indexes optimization queries
- [ ] Implement upsert logic for thumbnail updates using SeaORM's `on_conflict`

#### Step 9: Repository Integration
- [ ] Add repository to main database manager
- [ ] Create database connection pool management
- [ ] Implement transaction support for batch operations
- [ ] Add database migration verification

**Files to Create/Modify:**
- `/src-tauri/src/database/thumbnail_repository.rs` - New repository
- `/src-tauri/src/database/mod.rs` - Export repository
- `/src-tauri/src/database/operations.rs` - Integration

### 📋 Phase 3: Background Processing Pipeline  
**Goal**: Implement queue-based thumbnail generation with progress tracking

#### Step 10: Processing Queue Architecture
- [ ] Create `ThumbnailProcessor` struct for coordinating generation
- [ ] Implement work queue using `tokio::sync::mpsc` channels
- [ ] Design `ThumbnailJob` struct with file metadata and priority
- [ ] Add job status tracking: `Pending`, `Processing`, `Completed`, `Failed`

#### Step 11: Batch File Processing
- [ ] Implement file discovery and queuing:
  - `queue_files_for_processing(file_ids, sizes) -> Result<usize>`
  - `process_batch(batch_size: usize) -> Result<ProcessingStats>`
- [ ] Add concurrent processing with configurable worker count
- [ ] Implement memory management (process files in chunks to avoid OOM)
- [ ] Add progress reporting via Tauri events

#### Step 12: Error Recovery & Resilience
- [ ] Implement retry logic for failed thumbnail generation
- [ ] Add dead letter queue for persistently failing files
- [ ] Create cleanup jobs for orphaned thumbnails
- [ ] Add processing metrics and logging

**Files to Create:**
- `/src-tauri/src/file_system/thumbnail_processor.rs` - Main processor
- `/src-tauri/src/file_system/processing_queue.rs` - Queue management
- `/src-tauri/src/file_system/processing_stats.rs` - Metrics tracking

### 📋 Phase 4: Tauri IPC Integration
**Goal**: Expose thumbnail functionality to frontend via type-safe commands

#### Step 13: Tauri Command Implementation  
- [ ] Create thumbnail-specific commands in `/src-tauri/src/commands/thumbnail_commands.rs`:
  - `generate_thumbnail_for_file(file_id, size) -> Result<String>` (returns base64)
  - `get_thumbnail_by_file_id(file_id, size) -> Result<Option<String>>`
  - `generate_thumbnails_batch(file_ids, sizes) -> Result<u64>`
  - `get_thumbnail_generation_progress() -> Result<ProcessingStats>`

#### Step 14: Frontend Data Structures
- [ ] Create TypeScript interfaces in frontend:
  - `ThumbnailSize` enum matching Rust
  - `Thumbnail` interface with base64 data
  - `ProcessingStats` for progress tracking
- [ ] Implement frontend thumbnail cache management
- [ ] Add lazy loading support for thumbnail grids

#### Step 15: Event System Integration
- [ ] Implement Tauri events for real-time updates:
  - `thumbnail-generated` - Single thumbnail completion
  - `batch-progress-update` - Batch processing progress
  - `thumbnail-error` - Generation failures
- [ ] Add frontend event listeners for UI updates
- [ ] Implement thumbnail invalidation on file changes

**Files to Create/Modify:**
- `/src-tauri/src/commands/thumbnail_commands.rs` - New commands
- `/src-tauri/src/commands/mod.rs` - Export commands
- `/src-tauri/src/main.rs` - Register commands
- Frontend TypeScript interfaces and event handlers

### 📋 Phase 5: Performance Optimization & Caching
**Goal**: Optimize thumbnail pipeline for production performance

#### Step 16: Memory Optimization
- [ ] Implement streaming thumbnail generation for large files
- [ ] Add configurable memory limits per processing worker
- [ ] Implement thumbnail data compression before database storage
- [ ] Add memory usage monitoring and alerts

#### Step 17: Caching Strategy
- [ ] Implement in-memory LRU cache for frequently accessed thumbnails
- [ ] Add filesystem cache for thumbnail data (optional)
- [ ] Create cache invalidation strategy on file updates
- [ ] Add cache warming for newly imported files

#### Step 18: Performance Monitoring
- [ ] Add detailed performance metrics:
  - Generation time by file type and size
  - Database query performance
  - Memory usage per worker
  - Queue processing throughput
- [ ] Implement performance alerts and thresholds
- [ ] Create performance tuning configuration

**Files to Create:**
- `/src-tauri/src/file_system/thumbnail_cache.rs` - Caching layer
- `/src-tauri/src/file_system/performance_monitor.rs` - Metrics
- `/src-tauri/src/config/thumbnail_config.rs` - Configuration

### 📋 Phase 6: Advanced Features & Polish
**Goal**: Add production-ready features and user experience improvements

#### Step 19: Advanced Thumbnail Features
- [ ] Implement video thumbnail generation (first frame extraction)
- [ ] Add PDF thumbnail generation (first page)
- [ ] Create document preview thumbnails (LibreOffice/text files)
- [ ] Add thumbnail quality configuration per file type

#### Step 20: User Experience Features
- [ ] Implement thumbnail regeneration commands
- [ ] Add bulk thumbnail deletion
- [ ] Create thumbnail size management (disk usage)
- [ ] Add thumbnail export functionality

#### Step 21: Testing & Documentation
- [ ] Create comprehensive integration tests
- [ ] Add performance benchmarks
- [ ] Write user documentation for thumbnail features
- [ ] Create troubleshooting guide

**Files to Create:**
- `/src-tauri/src/file_system/advanced_generators.rs` - Video/PDF support
- `/tests/integration/thumbnail_pipeline_tests.rs` - Integration tests
- `/docs/thumbnail_system.md` - Documentation

## Technical Architecture

### Data Flow
```
File Discovery -> Queue -> Thumbnail Generator -> Database -> Frontend
     ↓              ↓           ↓                    ↓         ↓
File Scanner -> Processing -> Image Processing -> Repository -> UI
               Queue                                           ↓
                                                          Event Updates
```

### Performance Targets
- **Batch Processing**: 100+ files/second for small thumbnails
- **Memory Usage**: <500MB for 1000 concurrent thumbnails
- **Database**: <50ms query time for thumbnail retrieval
- **UI Responsiveness**: <100ms thumbnail display

### Configuration Points
- Worker thread count (default: CPU cores)
- Batch size (default: 50 files)
- Memory limits per worker (default: 100MB)
- Cache size (default: 1000 thumbnails)
- Thumbnail quality settings per format

## Next Session Priorities

1. **Start with Phase 2, Step 7**: Implement `ThumbnailRepository` 
2. **Database operations**: Focus on clean SeaORM integration
3. **Testing**: Verify repository operations work correctly
4. **Integration**: Connect repository to existing file system

## Notes for Tomorrow

- The SeaORM integration is working well - continue with this pattern
- Focus on repository pattern implementation first
- Test database operations before moving to background processing
- Consider adding database connection pooling early
- Remember to update existing file scanning to trigger thumbnail generation

Good night! 🌙