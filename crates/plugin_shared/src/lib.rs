use serde::Serialize;

#[derive(Serialize)]
pub struct Datasource {
    pub id: String,
}

#[derive(Serialize)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    pub exported_datasources: Vec<Datasource>,
}
