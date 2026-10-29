// The NDK media functions live in libmediandk.so; ndk-sys only links it on request.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "android") {
        println!("cargo:rustc-link-lib=mediandk");
    }
}
