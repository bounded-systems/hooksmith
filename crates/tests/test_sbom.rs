use std::process::Command;

fn main() {
    let output = Command::new("./target/debug/xtask")
        .arg("sbom")
        .arg("generate")
        .output()
        .expect("Failed to execute command");

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}
