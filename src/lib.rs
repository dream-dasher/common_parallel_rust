/*!
# ____________________
(default package of workspace)
*/

pub mod hidden_value {
    pub use utilities::{HiddenValue, HiddenValueError};
}

pub fn func_in_default_package_of_workspace() {
    println!("Hello from (a function defined in) the default package of the workspace.");
}
