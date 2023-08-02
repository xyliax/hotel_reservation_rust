# hotel_reservation_rust
Rust micro-service experimental architecture

## Services (services/*)
- [x] user
- [x] recommendation 
    - research, rate, profile
- [x] research
    - geo
- [x] rate
- [x] profile
- [x] geo
- [ ] reservation (Not used!)

## Commons (commons/*)
- [x] commons
    - Store micro-service configuration
    - Import common modules, symbols...
    - Common dependency
- [x] db_config
    - Config database constants (database URL, names, collections...)
    - tracing
    - interceptors

## Tester (test_worker)
- Issue request to user-service and recommendation-service



