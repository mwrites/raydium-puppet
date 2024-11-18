test-ts:
    anchor test

wbuild:
    RUST_BACKTRACE=1 cargo watch -x fmt -x "clippy --fix" -x build

wtest:
    RUST_BACKTRACE=1 cargo watch -x clippy -x "test -p raydium-client -- --nocapture"

test:
    RUST_BACKTRACE=1 cargo test --tests -- --show-output

test-both:
    RUST_BACKTRACE=1 cargo test --tests test_add_remove_liquidity -- --nocapture

test-add:
    RUST_BACKTRACE=1 cargo test --tests test_add_liquidity -- --nocapture

test-remove:
    RUST_BACKTRACE=1 cargo test --tests test_remove_liquidity -- --nocapture

check:
    cargo fmt && cargo clippy

fix:
    cargo fmt && cargo clippy --fix

mine:
    devnet-pow mine
