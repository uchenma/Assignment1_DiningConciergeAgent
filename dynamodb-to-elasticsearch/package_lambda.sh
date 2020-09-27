sudo docker run --rm -it -v "$(pwd)/../":/home/rust/src ekidd/rust-musl-builder /bin/bash -c "cd dynamodb-to-elasticsearch && cargo build --release"

cp ./target/x86_64-unknown-linux-musl/release/dynamodb-to-elasticsearch ./bootstrap
zip lambda.zip ./bootstrap
rm ./bootstrap
