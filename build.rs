use std::path::{PathBuf};
use bootloader::BootConfig;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let uefi_path = out_dir.join("phosphorOS_uefi.img");

    let uefi_boot = bootloader::UefiBoot::new(&kernel);

    uefi_boot.create_disk_image(&uefi_path).unwrap();

    // let bios_path = out_dir.join("bios.img");
    // bootloader::BiosBoot::new(&kernel).create_disk_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    // println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}