pub mod docker_source;
pub mod file_source;
pub mod source;

pub fn get_sources() -> Vec<Box<dyn source::Source>> {
    vec![
        Box::new(file_source::FileSource {}),
        Box::new(docker_source::DockerSource {}),
    ]
}
