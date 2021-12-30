use crate::PathOrURI;

const OPEN_COMMAND: &str = "open";

pub(crate) fn open(target: &PathOrURI) -> crate::Result {
    crate::open_with_command(OPEN_COMMAND, target)
}
