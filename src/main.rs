use formulafix::{discard_workflow, get_args, pull_back_workflow, take_backup};

#[tokio::main]
async fn main() {
    println!("================ formulafix ================");
    let args = get_args();
    println!("─➤ Arguments provided: {:#?}", args);
    let db = formulafix::get_db_connection(args.conn()).await;
    if db.is_err() {
        eprintln!("─➤ Not able to connect to database!");
        std::process::exit(1);
    }
    let db = db.unwrap();
    take_backup(&args, &db).await;
    if args.ret() {
        println!("─➤ Pull back workflow {}", args.work_flow_id());
        let _ = pull_back_workflow(&args, &db).await;
    } else if args.discard() {
        println!("─➤ Discard workflow {}", args.work_flow_id());
        let _ = discard_workflow(&args, &db).await;
    }
    println!("─➤ Completed Successfully!!!");
}
