use anyhow::Result;
use clap::Args;

// ============================================
// TESTS
// ============================================
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestArgs {
        #[command(flatten)]
        init: InitArgs,
    }

    #[test]
    fn test_init_args_parsing() {
        let args = TestArgs::parse_from(["program"]);
        // InitArgs is empty, just verify it parses
        let _init = args.init;
    }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

#[derive(Args, Debug)]
pub struct InitArgs {
    // Init command takes no arguments
}

// ============================================
// IMPLEMENTATIONS
// ============================================

pub fn run(_args: InitArgs) -> Result<()> {
    crate::init::run(None)
}
