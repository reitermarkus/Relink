fn main() {
    println!("cargo:rustc-link-arg=-Wl,--emit-relocs");
    println!("cargo:rustc-link-arg=-Wl,-rpath");
    println!("cargo:rustc-link-arg=$ORIGIN");
}
