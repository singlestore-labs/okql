wit_bindgen_rust::export!("kql_to_sql.wit");

use converter::kql_to_sql;

struct KqlToSql;

impl kql_to_sql::KqlToSql for KqlToSql {
    fn convert(kql: String) -> Result<String, String> {
        kql_to_sql("test.kql".into(), kql)
    }
}
