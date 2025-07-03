// Slightly hacky build script that gets sad if you're building in github actions with a tag that doesn't match the crate's version
fn main() {
    if let Some(rf) = option_env!("GITHUB_REF")
        && let Some(cargo_ver) = option_env!("CARGO_PKG_VERSION")
        && let Some(tag) = rf.strip_prefix("refs/tags/")
        && tag != cargo_ver
    {
        println!("cargo::error=Github tag {tag} doesn't match cargo manifest version {cargo_ver}!")
    }
}
