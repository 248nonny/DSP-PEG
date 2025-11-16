fn main() {
    println!("cargo:rerun-if-changed=baremetal/linker.ld");
    println!("cargo:rustc-link-arg=-Tbaremetal/linker.ld");
}
