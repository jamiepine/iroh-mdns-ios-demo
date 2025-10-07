fn main() {
    // Link Apple frameworks when building for iOS
    #[cfg(target_os = "ios")]
    {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}
