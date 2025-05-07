// build.rs

// https://docs.rs/build-data/0.1.3/build_data/
fn main() -> anyhow::Result<()> {
    let _ = build_data::set_GIT_BRANCH();
    let _ = build_data::set_GIT_COMMIT();
    let _ = build_data::set_SOURCE_TIMESTAMP();
    let _ = build_data::set_RUSTC_VERSION();
    let _ = build_data::no_debug_rebuilds();
    Ok(())
}
// EOF
