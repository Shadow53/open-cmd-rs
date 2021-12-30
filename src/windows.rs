use crate::PathOrURI;
use std::process::Command;

pub(crate) fn open(target: &PathOrURI) -> crate::Result {
    crate::ensure_command("cmd")?;

    #[cfg(feature = "tracing")]
    tracing::debug!("opening {} with default Windows handler", target);

    let mut cmd = Command::new("cmd");
    cmd.args(&["/c", "start", target.uri()?.to_string().as_str()]);
    Ok(cmd)
}
