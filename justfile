test:
  cargo test -- --nocapture
bench: 
  cargo bench
coverage:
  cargo tarpaulin --out html -- --nocapture
doit:
  cargo build && sudo ./target/debug/theclicker
