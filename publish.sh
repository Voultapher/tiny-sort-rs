#!/usr/bin/env bash

set -euxo pipefail

NEW_VERSION="${1}"

# Ensure Cargo.toml has been updated.
CARGO_TOML_VERSION=$(grep "${NEW_VERSION}" Cargo.toml)

# Verify we are on main
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "${CURRENT_BRANCH}" != "main" ]; then
    echo "Not on main"
    exit 1
fi

git fetch origin

DIFF_TO_ORIGIN=$(git diff origin/main)
if [ "${DIFF_TO_ORIGIN}" != "" ]; then
    echo "Out of sync with origin/main"
    exit 1
fi

cargo test
cargo miri nextest run --verbose --target x86_64-unknown-linux-gnu -j32
cargo miri test --target mips64-unknown-linux-gnuabi64 --test stable random_z1
cargo miri test --target mips64-unknown-linux-gnuabi64 --test unstable random_z1

git tag "v${NEW_VERSION}"
git push --tags

cargo publish