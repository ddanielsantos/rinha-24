pub struct Config<'a> {
    pub database_url: &'a str,
}

pub fn get_config<'a>() -> Config<'a> {
    todo!()
}