use clap::Parser;

use converter::kql_to_sql;

#[derive(Parser, Debug)]
struct Arguments {
    input: String,
}

fn main() {
    let Arguments { input: kql } = Arguments::parse();
    println!("KQL: {}", kql);
    match kql_to_sql("input.kql".into(), kql) {
        Ok(sql) => println!("SQL:\n{}", sql),
        Err(error) => println!("Errors:\n{}", error),
    }
}
