
pub struct Ssm2 {
    pub port_name: String,
}

impl Ssm2 {
    pub fn new(port_name: String) -> Self {
        Ssm2 { port_name }
    }
}