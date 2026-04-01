use testcontainers::core::{CmdWaitFor, ExecCommand};
use testcontainers::{runners::SyncRunner, GenericImage};
use std::env;
use std::process::Command;

fn build_images() {
    let cwd = env::var("CARGO_MANIFEST_DIR").unwrap();
    let _out = Command::new("docker")
        .arg("compose")
        .arg("build")
        .current_dir(format!("{cwd}/docker/"))
        .output()
        .expect(&format!("Failed to execute command. Check directory {}", cwd));
}

fn run_command(args: &str, wait_for: &str) -> (String, String) {
    let custom_image = GenericImage::new("test_image", "latest");
    let container = custom_image.start().unwrap();

    let args_array: Vec<&str> = args.split_whitespace().collect();

    let mut res = container
        .exec(
            ExecCommand::new(args_array)
                .with_cmd_ready_condition(CmdWaitFor::message_on_stderr(wait_for))
                .with_cmd_ready_condition(CmdWaitFor::seconds(10)),
        )
        .unwrap_or_else(|e| {
            panic!("Failed to run cmd {}\nError:\n{:?}", args, e.to_string());
        });

    let out = String::from_utf8(res.stdout_to_vec().unwrap()).unwrap();
    let err = String::from_utf8(res.stderr_to_vec().unwrap()).unwrap();

    (out, err)
}

#[test]
fn test_ip_assigning() {
    // If Docker is not available (e.g. in lightweight CI or dev machines),
    // skip this integration test instead of failing.
    if !std::path::Path::new("/var/run/docker.sock").exists() {
        eprintln!("Skipping DHCP integration test: Docker socket not found");
        return;
    }

    build_images();

    let client_thread = std::thread::spawn(move || {
        let (_out, err) = run_command("dhclient -4 -d -v -p 6768", "bound to");

        let expected_lines = [
            "binding to user-specified port",
            "DHCPDISCOVER on",
            "bound to",
        ];

        for expected in &expected_lines {
            assert!(err.contains(expected),
                "Expected line not found: {}\nCheck on the complete logs:\n{}", expected, err);
        }
    });

    let server_thread = std::thread::spawn(move || {
        let (out, _err) = run_command("quick-serve --dhcp=6767 -v --bind-ip=172.12.1.4", "dhcp_server: offered");

        let expected_lines = [
            "DHCP server serving on",
            "dhcp_server: Request received",
            "dhcp_server: offered",
        ];

        for expected in &expected_lines {
            assert!(out.contains(expected),
                "Expected line not found: {}\nCheck on the complete logs:\n{}", expected, out);
        }
    });

    client_thread.join().unwrap();
    server_thread.join().unwrap();
}
