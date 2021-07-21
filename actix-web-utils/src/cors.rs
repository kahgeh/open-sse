use actix_cors::Cors;

pub fn create_cors_policy(allowed_origin: &str) -> Cors{
    let cors=Cors::default()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

    if allowed_origin == "*" {
        cors.allow_any_origin()
    } else {
        cors.allowed_origin(allowed_origin)
    }
}