use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn send_not_enough_args() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("send").arg("192.168.1.32");
    cmd.assert().failure().stderr(predicate::str::contains(
        "The following required arguments were not provided",
    ));
    Ok(())
}

#[test]
fn send_to_malformed_ipv4_group() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("send")
        .arg("192.168.3.32")
        .arg("4001")
        .arg("bad.ip.address")
    ;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse group address"));
    Ok(())
}

#[test]
fn send_to_out_of_range_port() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("send")
        .arg("192.168.3.32")
        .arg("65537")
        .arg("231.0.3.1")
    ;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse port number"));
    Ok(())
}

#[test]
fn send_to_malformed_ipv4_interface() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("send")
        .arg("192324.168.3.32")
        .arg("4001")
        .arg("231.0.3.1")
    ;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse nic address"));
    Ok(())
}

#[test]
fn listen_not_enough_args() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("listen").arg("192.168.3.32");
    cmd.assert().failure().stderr(predicate::str::contains(
        "The following required arguments were not provided",
    ));
    Ok(())
}

#[test]
fn listen_to_malformed_ipv4_group() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("listen")
        .arg("192.168.3.32")
        .arg("4001")
        .arg("bad.ip.address")
    ;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse group address"));
    Ok(())
}

#[test]
fn listen_to_out_of_range_port() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("listen")
        .arg("192.168.3.32")
        .arg("65537")
        .arg("231.0.3.1")
    ;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse port number"));
    Ok(())
}

#[test]
fn listen_to_malformed_ipv4_interface() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("listen")
        .arg("192324.168.3.32")
        .arg("4001")
        .arg("231.0.3.1")
    ;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse nic address"));
    Ok(())
}

#[test]
fn unrecognised_command() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("nonexistentcmd");
    cmd.assert().failure();
    Ok(())
}
