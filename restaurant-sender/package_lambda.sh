sudo docker run --rm -it -v "$(pwd)/../":/home/rust/src ekidd/rust-musl-builder:nightly-2020-08-26 /bin/bash -c "cd restaurant-sender && cargo build --release"

cp ./target/x86_64-unknown-linux-musl/release/restaurant-sender ./bootstrap
zip lambda.zip ./bootstrap
rm ./bootstrap
