# TODO List

## Completed Tasks (2025-08-10)
- [x] Search for TODO comments in the codebase
- [x] Analyze project structure and identify main components
- [x] Search for common bug patterns
- [x] Implement TODO: wait for tasks to shutdown in src/lib.rs:177
- [x] Run tests to verify fixes
- [x] Fix deprecated warnings and improve code quality
- [x] Check for unchecked unwraps that could panic
- [x] Modernize to Rust 2018 edition
- [x] Add comprehensive rustdoc documentation
- [x] Update project_description.md with current status
- [x] Identify all functions requiring unit tests

## Recently Completed Tasks (2025-08-10 Session)
- [x] Create test module for build_txt_record() function - 7 comprehensive tests
- [x] Test ServicesInner registry operations (register/unregister/find) - 8 tests
- [x] Test service discovery by name and type
- [x] Test error conditions and panic scenarios
- [x] Create integration tests for service lifecycle - 6 tests
- [x] Fix clippy warning (FSM → Fsm naming)
- [x] All 22 tests passing successfully (15 unit + 6 integration + 1 doc)

## Next Priority Tasks

### Advanced Testing
- [ ] Test actual mDNS packet generation and parsing
- [ ] Add property-based testing with quickcheck
- [ ] Test network failure scenarios
- [ ] Add benchmarks for performance critical paths

## Future Tasks

### Code Improvements
- [ ] Add tests for CommandSender message dispatch
- [ ] Add tests for Shutdown mechanism with mock threads
- [ ] Test ServiceData record builders (PTR/SRV/TXT)
- [ ] Add tests for Fsm state machine behavior

### Error Handling
- [ ] Replace remaining `.expect()` calls with proper error handling where appropriate
- [ ] Add custom error types instead of using generic io::Error
- [ ] Improve error messages for better debugging

### Code Quality
- [ ] Update to Rust 2021 edition
- [ ] Add GitHub Actions CI/CD pipeline
- [ ] Add code coverage reporting

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