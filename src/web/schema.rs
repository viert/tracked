use avro_rs::Schema;
use lazy_static::lazy_static;

lazy_static! {
  pub static ref SCHEMAS: Vec<Schema> = {
    let tp_schema = r#"
    {
      "type": "record",
      "name": "TrackPoint",
      "fields": [
        { "name": "ts", "type": "int" },
        { "name": "la", "type": "double" },
        { "name": "lo", "type": "double" },
        { "name": "h", "type": "int" },
        { "name": "g", "type": "int" },
        { "name": "a", "type": "int" }
      ]
    }
    "#;
    let tpl_schema = r#"
    {
      "type": "array",
      "name": "TrackPointList",
      "items": "TrackPoint"
    }
    "#;
    let schemas = Schema::parse_list(&[tp_schema, tpl_schema]).unwrap();
    schemas
  };
}
