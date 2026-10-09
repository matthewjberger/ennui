set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

dynamic := "on"
tool := "cargo run -q --release --manifest-path tools/ennui/Cargo.toml --"

default:
    @just --list

# Run an app by package name: the gallery, the editor on the template's project, or an example such as showcase
run app *args:
    @{{tool}} run --dynamic {{dynamic}} {{app}} -- {{args}}

# Run with Chrome tracing, to measure where the frame time goes
trace app *args:
    @{{tool}} run --dynamic {{dynamic}} --trace {{app}} -- {{args}}

# Build apps for the browser into target/site: the first is the main page, with a button to each of the others, such as gallery showcase
web +apps:
    @{{tool}} web {{apps}}

# Build apps for the browser and serve target/site at http://127.0.0.1:8080
serve +apps:
    @{{tool}} web --serve 8080 {{apps}}

# Pick an app from a list and run it
pick *query:
    @{{tool}} pick --dynamic {{dynamic}} --folder apps {{query}}

# Make a new app repository at <folder>: a copy of template/ with its names changed and its paths pointed at this clone, synced, with a first commit
new folder:
    @{{tool}} new --from '{{invocation_directory_native()}}' '{{folder}}'

# Format the crates, the apps and the ennui tool
fmt:
    cargo fmt --all
    cargo fmt --manifest-path tools/ennui/Cargo.toml

# Check the format, then run clippy on the crates, the apps and the ennui tool
lint:
    cargo fmt --all -- --check
    cargo fmt --manifest-path tools/ennui/Cargo.toml -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy --manifest-path tools/ennui/Cargo.toml --all-targets -- -D warnings
