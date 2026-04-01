use utoipa::openapi::{
    path::PathsBuilder,
    schema::ComponentsBuilder,
    InfoBuilder,
    OpenApiBuilder,
};

pub fn build_openapi(service_name: &str, version: &str) -> utoipa::openapi::OpenApi {
    OpenApiBuilder::new()
        .info(InfoBuilder::new().title(service_name).version(version).build())
        .paths(PathsBuilder::new().build())
        .components(Some(ComponentsBuilder::new().build()))
        .build()
}
