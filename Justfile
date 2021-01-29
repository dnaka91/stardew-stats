build:
    trunk clean
    trunk build --release
    for file in `ls dist/*.wasm`; do \
        wasm-opt -O -o $file $file; \
    done

watch:
    cargo watch -s 'just build' -s 'miniserve dist --index index.html'