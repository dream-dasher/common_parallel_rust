/*!
# ____________________
(default package of workspace)
*/

use common_parallel_rust::{func_in_default_package_of_workspace, hidden_value};

fn main() {
    println!("Hello from the default package of the workspace.");
    func_in_default_package_of_workspace();

    // Demonstrate usage of hidden_value from the default package
    // Set a test environment variable
    unsafe {
        std::env::set_var("TEST_SECRET", "my_secret_value");
    }

    let secret = hidden_value::HiddenValue::new_from_env("TEST_SECRET", false, None)
        .expect("Failed to create hidden value");
    println!("Hidden value created: {:?}", secret);
    println!("Exposed value: {}", secret.expose_value());
}
