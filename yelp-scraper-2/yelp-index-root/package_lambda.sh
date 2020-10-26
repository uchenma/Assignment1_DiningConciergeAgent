rm ./lambda.zip

docker run --rm -it -v "$(pwd)/../":/home/rust/src ekidd/rust-musl-builder:nightly-2020-08-26 /bin/bash -c "cd yelp-index-root && cargo build --release"

cp ./target/x86_64-unknown-linux-musl/release/yelp-index-root ./bootstrap
zip lambda.zip ./bootstrap
rm ./bootstrap
