use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AssertionAction {
    List,
    Enable {
        index: usize,
    },
    Disable {
        index: usize,
    },
    Toggle {
        index: usize,
    },
    Add {
        check: String,
        #[arg(long, default_value = "stop")]
        on_fail: String,
    },
    Remove {
        index: usize,
    },
}
