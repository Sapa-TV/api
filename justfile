gen:
    cargo test -- --nocapture generate_openapi

# GET endpoints
get-king:
    curl -s http://localhost:3000/api/king

get-month:
    curl -s http://localhost:3000/api/month

get-last-day:
    curl -s http://localhost:3000/api/last-day

# POST endpoints
post-king name:
    curl -s -X POST http://localhost:3000/api/king \
        -H "Content-Type: application/json" \
        -d '{"name": "{{name}}"}'

post-month name:
    curl -s -X POST http://localhost:3000/api/month \
        -H "Content-Type: application/json" \
        -d '{"name": "{{name}}"}'

post-last-day name:
    curl -s -X POST http://localhost:3000/api/last-day \
        -H "Content-Type: application/json" \
        -d '{"name": "{{name}}"}'