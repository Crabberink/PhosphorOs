fn main() {

    let uefi_path = env!("UEFI_PATH");
    // let bios_path = env!("BIOS_PATH");

    let uefi = true;

    println!("{}",uefi_path);

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        println!("{}",ovmf_prebuilt::ovmf_pure_efi().display());
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        println!("{}",format!("format=raw,file={uefi_path}"));
        cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
        cmd.arg("-serial").arg("stdio");
        // cmd.arg("-nographic");
        // cmd.arg("-s");
    } else {
        // cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
