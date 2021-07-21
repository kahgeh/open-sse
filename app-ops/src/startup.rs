pub struct ApplicationStartUpDisplayInfo {
    pub environment_name: String,
    pub is_debug: String,
    pub port: String,
}

impl ApplicationStartUpDisplayInfo {
    pub fn new(environment_name: &str, is_debug: bool, port: u16 )->ApplicationStartUpDisplayInfo{
        ApplicationStartUpDisplayInfo{
            environment_name: String::from(environment_name),
            is_debug : match is_debug { true => String::from("true"), _ => String::from("false") },
            port: port.to_string(),
        }
    }
}