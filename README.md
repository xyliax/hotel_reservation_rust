# Hotel Reservation in Rust
Rust micro-service experimental architecture

## Environment
### Linux
Development Version
``` bash
Linux 5.15.0-56-generic x86_64 GNU/Linux Ubuntu 22.04.2 LTS
Intel Xeon E3-1280 v6, 4x Sky Lake, 8x Logical Processors
```
- Rust
- Cargo
- Mongodb

## Build and Run
### Build
To build the project with Cargo
``` shell
cargo
```


## Package Structure
### Services (services/*)
- [x] user
- [x] recommendation 
    - research, rate, profile
- [x] research
    - geo
- [x] rate
- [x] profile
- [x] geo
- [ ] reservation [Not used!]

### Commons (commons/*)
- [x] commons
    - Store micro-service configuration
    - Import common modules, symbols...
    - Common dependency
- [x] db_config
    - Config database constants (database URL, names, collections...)
    - tracing
    - interceptors

### Tester (test_worker)
- Issue request to user-service and recommendation-service



