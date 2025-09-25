# Thumbnail Caching and Preloading Strategy

## Overview
This document outlines the implementation strategy for improving thumbnail performance through in-memory caching and intelligent preloading, based on analysis of Redis vs SurrealDB options and current architecture.

## Decision: In-Memory Caching + Preloading
**Chosen approach:** LRU in-memory cache with adjacent file preloading
**Rejected:** External KV stores (Redis/SurrealDB) - unnecessary complexity for desktop app

## Implementation Phases

### Phase 1: LRU Memory Cache (Week 1) ⭐ **START HERE**

#### Dependencies
```toml
# Add to Cargo.toml
moka = { version = "0.12", features = ["future"] }
```

#### Core Components
- **ThumbnailMemoryCache**: LRU cache with TTL/TTI eviction
- **Cache Configuration**: 1000 thumbnails max (~50-100MB memory)
- **Cache Integration**: Layer between Tauri commands and database

#### Key Benefits
- Sub-millisecond retrieval for cached thumbnails
- Automatic memory management with LRU eviction
- 5-minute idle timeout, 30-minute total lifetime

### Phase 2: Adjacent File Preloading (Week 2) ⭐ **HIGH IMPACT**

#### Strategy
When user views a file, preload thumbnails for ±2 adjacent files in the same folder.

#### Why This Strategy First?
- **Highest user value**: 70%+ of browsing is sequential 
- **Predictable behavior**: Users typically go from file X to X±1
- **Simple implementation**: No complex state tracking
- **Low resource cost**: Only 2-4 extra thumbnails per view

#### Implementation
```rust
// Preload adjacent files when user views current file
preloader.preload_adjacent_files(current_file_id, 2).await
```

### Phase 3: Smart Folder Preloading (Week 3-4)

#### Strategy  
When user opens a folder, preload first 20-30 thumbnails prioritized by:
1. Image files first (more likely to need thumbnails)
2. Recently modified files
3. Alphabetical order

#### Benefits
- Instant folder navigation experience
- Works with existing folder browsing patterns
- Manageable system load

### Phase 4: Behavior Tracking (Future Enhancement)

#### Strategy
Track user patterns to predict which files/folders to preload.

#### Why Last?
- Most complex implementation
- Requires persistent behavior storage  
- Diminishing returns (adjacent + folder covers 80% of use cases)
- Privacy considerations for user data

## Architecture Integration

### Current Flow
```
Frontend ↔ Tauri IPC ↔ Rust Backend ↔ SQLite Database
```

### Enhanced Flow  
```
Frontend ↔ Tauri IPC ↔ Rust Backend ↔ [Memory Cache] ↔ SQLite Database
                                           ↑
                                    [Preloader Service]
```

### Modified Components
- **Tauri Commands**: Add cache layer before database queries
- **AppState**: Include cache and preloader services
- **Background Service**: Continuous preloading based on user behavior

## Performance Expectations

### Before (Current)
- Database query: ~5-50ms per thumbnail
- Cold thumbnail generation: 100-500ms
- Folder browsing: Multiple database hits

### After (With Cache + Preloading)
- Cache hit: <1ms retrieval
- Folder navigation: Instant (preloaded)
- Reduced CPU: 70% fewer regenerations
- Better UX: Smooth, responsive browsing

## Implementation Notes

### Memory Management
- **Cache Size**: 1000 thumbnails max
- **Eviction**: LRU with time-based expiration
- **Memory Usage**: ~50-100MB typical
- **Cleanup**: Automatic via moka library

### Integration with Existing Code
- **ThumbnailProcessor**: Enhanced with cache integration
- **File Operations**: Cache invalidation on file changes  
- **Repository Layer**: Fallback when cache misses

### Monitoring & Debugging
- Cache hit/miss ratios
- Memory usage tracking
- Preloading effectiveness metrics
- Performance benchmarking

## Success Metrics

### Technical Metrics
- Cache hit ratio >70%
- Average thumbnail retrieval <5ms
- Memory usage <150MB
- 50% reduction in database queries

### User Experience Metrics  
- Perceived folder load time <200ms
- Smooth scrolling through file lists
- Reduced thumbnail loading delays
- Better responsiveness during browsing

## Future Considerations

### Scalability
- Cache size adjustment based on available system memory
- Intelligent preloading based on user patterns
- Cross-session cache persistence (optional)

### Extensions
- Thumbnail prefetching during idle time
- Network-aware caching for cloud storage
- User-configurable cache settings

## Implementation Priority

1. **Week 1**: Basic LRU memory cache
2. **Week 2**: Adjacent file preloading (±2 files)  
3. **Week 3-4**: Smart folder preloading (first 30 files)
4. **Future**: Behavior tracking and advanced prediction

Start with Phase 1 & 2 for maximum impact with minimum complexity.