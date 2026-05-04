gen:
    cargo test -- --nocapture generate_openapi

local:
    cargo test -- --nocapture generate_openapi
    cp generated/openapi.json ../api-contracts/openapi.json

res:
    cargo run --bin research
