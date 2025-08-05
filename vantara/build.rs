fn main() {
    println!("cargo:rustc-link-search=native=/opt/musl-lzma/lib");
    println!("cargo:rustc-link-lib=static=lzma");
}
