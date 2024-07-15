# rust image proocessing server to compress image on the fly

## getting startted locally
```
cargo run
# or if you want to watch for changes
cargo watch -x run
```

## how does it work
`http://127.0.0.1:8080/?url=<image-url>&quality=<quality>`
the quality must be between 1 and 100

### Currently supported
[X] compress image
[] change image format