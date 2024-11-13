use clap::Parser;

#[derive(Parser, Debug)]
#[command(about)]
pub(crate) struct Args {
    #[arg(long, short, default_value = "0.0.0.0")]
    pub(crate) host: String,
    #[arg(long, short, default_value_t = 53)]
    pub(crate) port: u16,
    #[arg(long, short = 'P', default_value = "udp")]
    pub(crate) proto: String,
    #[arg(long, short, default_value = "5s")]
    pub(crate) timeout: String,
    #[arg(long, short, value_parser, num_args = 1.., value_delimiter = ',')]
    pub(crate) forward: Option<Vec<String>>,
    #[arg(long, short, default_value_t = 53)]
    pub(crate) default_forward_port: u16,
    #[arg(long, short, default_value = None)]
    pub(crate) zones: Option<String>,
    #[arg(long, short = 'N', default_value_t = false)]
    pub(crate) nested_zones: bool,
    #[arg(long, short = '6', default_value_t = false)]
    pub(crate) enable_ipv6: bool
}