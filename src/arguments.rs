use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    author = "Sibaprasad Maiti",
    version,
    about = "An utility to discard or return formula in IPA application."
)]
pub struct Arguments {
    #[clap(short = 'C', long = "conn")]
    conn: String,
    #[clap(short = 'D', long = "discard")]
    discard: bool,
    #[clap(short = 'P', long = "pull-back")]
    ret: bool,
    #[clap(short = 'S', long = "stage")]
    ret_stage: Option<u8>,
    #[clap(short = 'W', long = "workFlowId")]
    work_flow_id: u32,
    #[clap(short = 'F', long = "flowTypeId")]
    flow_type_id: Option<u8>,
    #[clap(short = 'M', long = "merge-coll")]
    merge_collection: Option<String>,
}

impl Arguments {
    #[cfg(test)]
    pub(crate) fn new(
        conn: String,
        discard: bool,
        ret: bool,
        ret_stage: Option<u8>,
        work_flow_id: u32,
        flow_type_id: Option<u8>,
        merge_collection: Option<String>,
    ) -> Self {
        Self {
            conn,
            discard,
            ret,
            ret_stage,
            work_flow_id,
            flow_type_id,
            merge_collection,
        }
    }
    pub fn conn(&self) -> &str {
        self.conn.as_str()
    }
    pub fn discard(&self) -> bool {
        self.discard
    }
    pub fn ret(&self) -> bool {
        self.ret
    }
    pub fn ret_stage(&self) -> Option<u8> {
        self.ret_stage
    }
    pub fn work_flow_id(&self) -> u32 {
        self.work_flow_id
    }
    pub fn flow_type_id(&self) -> u8 {
        self.flow_type_id.unwrap_or(1)
    }
    pub fn merge_collection(&self) -> String {
        if self.merge_collection.is_none() {
            "workItemMergeIpa".to_string()
        } else {
            self.merge_collection.as_ref().unwrap().clone()
        }
    }
}

/// parses command line arguments and return the `Arguments` type
pub fn get_args() -> Arguments {
    let mut args = Arguments::parse();
    // vaidate flowTypeId
    if args.flow_type_id() != 1
        && args.flow_type_id() != 2
        && args.flow_type_id() != 5
        && args.flow_type_id() != 6
    {
        eprintln!("Invalid --flowTypeId provided");
        std::process::exit(1);
    }

    // validate either discard or return flag must be present
    if !args.discard && !args.ret {
        eprintln!("Either --discard or --pull-back flag must be set");
        std::process::exit(1);
    }

    // validate return-stage value
    if args.ret {
        match args.ret_stage {
            Some(1) => {}
            Some(3) => {}
            _ => {
                eprintln!("Value of --stage must be 1 or 3");
                std::process::exit(1);
            }
        }
    }

    // override discard flag when return flag is received
    if args.ret {
        args.discard = false;
    }

    args
}
