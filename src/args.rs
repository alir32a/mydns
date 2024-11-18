use clap::Parser;

#[derive(Parser, Debug)]
#[command(about)]
pub(crate) struct Args {
    #[arg(long, short = 'H')]
    pub(crate) host: Option<String>,
    #[arg(long, short)]
    pub(crate) port: Option<u16>,
    #[arg(long, short = 'P')]
    pub(crate) proto: Option<String>,
    #[arg(long, short, default_value = "5s")]
    pub(crate) timeout: Option<String>,
    #[arg(long, short, value_parser, num_args = 1.., value_delimiter = ',')]
    pub(crate) forward: Option<Vec<String>>,
    #[arg(long, short)]
    pub(crate) default_forward_port: Option<u16>,
    #[arg(long, short, default_value_t = false)]
    pub(crate) authoritative: bool,
    #[arg(long, short)]
    pub(crate) zones: Option<String>,
    #[arg(long, short = 'N')]
    pub(crate) nested_zones: Option<bool>,
    #[arg(long, short = '6')]
    pub(crate) enable_ipv6: Option<bool>,
    #[arg(long, short = 'c')]
    pub(crate) config_file: Option<String>,
}