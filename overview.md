# Project Overview

## mDNS Responder Library

### Core Components

1. **lib.rs** - Main library interface
   - `Responder`: Main struct for mDNS operations
   - `Service`: Represents a registered service
   - `Shutdown`: Handles graceful shutdown with thread synchronization

2. **fsm.rs** - Finite State Machine for mDNS protocol
   - Handles incoming mDNS queries
   - Sends mDNS responses
   - Manages command processing

3. **services.rs** - Service registry
   - `ServicesInner`: Manages registered services
   - Service lookup by name and type
   - Service registration/unregistration

4. **net.rs / netwin.rs** - Network abstraction
   - Platform-specific socket handling
   - Multicast group management

5. **address_family.rs** - IPv4/IPv6 abstraction
   - Generic handling of different address families

### Key Features
- Asynchronous operation using tokio
- Thread-safe service registration
- Graceful shutdown with proper cleanup
- Cross-platform support (Unix/Windows)

### Dependencies
- tokio-core: Async runtime
- dns-parser: DNS packet parsing
- futures: Async primitives
- net2: Advanced socket options