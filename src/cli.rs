use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CLIData {
    #[structopt(name = "device", short, long, default_value = "default")]
    pub device_name: String,
    pub text_data_file: String,
}

impl CLIData {
    pub fn new() -> Self {
        Self::from_args()
    }
}
