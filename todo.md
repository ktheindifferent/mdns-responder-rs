# TODO List

## Completed Tasks
- [x] Search for TODO comments in the codebase
- [x] Analyze project structure and identify main components
- [x] Search for common bug patterns
- [x] Implement TODO: wait for tasks to shutdown in src/lib.rs:177
- [x] Run tests to verify fixes
- [x] Fix deprecated warnings and improve code quality
- [x] Check for unchecked unwraps that could panic

## Future Improvements

### Error Handling
- [ ] Replace remaining `.expect()` calls with proper error handling where appropriate
- [ ] Add custom error types instead of using generic io::Error
- [ ] Improve error messages for better debugging

### Code Quality
- [ ] Add unit tests for the shutdown mechanism
- [ ] Add integration tests for service registration/discovery
- [ ] Document public APIs with rustdoc comments
- [ ] Update to Rust 2021 edition

### Features
- [ ] Add support for service browsing
- [ ] Implement conflict resolution for duplicate service names
- [ ] Add metrics/monitoring capabilities
- [ ] Support for TXT record updates without re-registration

### Performance
- [ ] Optimize memory allocations in hot paths
- [ ] Consider using Arc for ServiceData to reduce cloning
- [ ] Profile and optimize DNS packet parsing

### Documentation
- [ ] Add comprehensive README with usage examples
- [ ] Create API documentation
- [ ] Add architectural diagrams