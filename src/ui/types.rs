#[derive(Clone, PartialEq)]
pub enum Screen {
    /// VMs List
    List,
    /// Confirmation popup when deleting a VM. The boolean value indicates if "OK" has been selected
    DeleteConfirmation(bool),
    /// Popup to show the error message when startnb.sh failed
    StartNbFailed {
        vm_name: String,
        error: String,
        stdout: String,
        stderr: String,
    },
    KillFailed {
        vm_name: String,
        error: String,
    },
}
