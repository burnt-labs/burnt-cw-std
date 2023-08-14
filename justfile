platform := if arch() =~ "aarch64" {"linux/arm64"} else {"linux/amd64"}
image := if arch() =~ "aarch64" {"cosmwasm/workspace-optimizer-arm64:0.12.12"} else {"cosmwasm/workspace-optimizer:0.12.12"}
optimize:
    docker run --rm -v "$(pwd)":/code --platform {{platform}} \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      {{image}}

unit-test:
    cargo test

lint:
	cargo +nightly clippy --all-targets -- -D warnings