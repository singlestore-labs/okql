use std::fs;

use converter::kql_to_sql;

use pretty_assertions::assert_eq;

#[test]
fn test_all() {
    let kql_files = fs::read_dir("./tests")
        .unwrap()
        .map(|path| path.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".kql"));

    for file_name in kql_files {
        println!("Testing '{}'", file_name);
        let kql_contents = fs::read_to_string(format!("./tests/{}", file_name)).unwrap();
        let stem = file_name.trim_end_matches(".kql");
        let sql_contents = fs::read_to_string(format!("./tests/{}.sql", stem)).unwrap();

        let result_sql = match kql_to_sql(file_name.clone(), kql_contents) {
            Ok(output) => output,
            Err(error) => {
                println!("{}", error);
                panic!("Failed to convert '{}'", file_name);
            },
        };

        assert_eq!(sql_contents, result_sql);
    }
}
