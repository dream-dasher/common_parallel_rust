/*!
# Async Processes
[Tokio Processes](https://docs.rs/tokio/latest/tokio/process/index.html)
*/
use tokio::process::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The usage is similar as with the standard library's `Command` type
    let mut child = Command::new("echo")
        .arg("hello")
        .arg("world")
        .spawn()
        .expect("failed to spawn");

    // Await until the command completes
    let status = child.wait().await?;
    println!("the command exited with: {}", status);
    let output = Command::new("echo").arg("hello").arg("world").output();

    let output = output.await?;

    dbg!(&output.status.success());
    dbg!(&output.stdout);
    dbg!(std::str::from_utf8(&output.stdout)?);

    Ok(())
}
