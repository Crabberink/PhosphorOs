use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

fn main() {
    let prebuilt = Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("failed to fetch prebuilt");

    let code_path = prebuilt.get_file(Arch::X64, FileType::Code);
    let vars_path = prebuilt.get_file(Arch::X64, FileType::Vars);

    let uefi_path = env!("UEFI_PATH");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    cmd.arg("-drive").arg(format!("if=pflash,format=raw,readonly=on,file={}", code_path.display()));
    cmd.arg("-drive").arg(format!("if=pflash,format=raw,file={}", vars_path.display()));

    cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));

    cmd.arg("-serial").arg("stdio");
    // cmd.arg("-nographic");
    // cmd.arg("-s");

    println!("Running: {:?}", cmd);

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
