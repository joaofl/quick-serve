use crate::servers::server::Protocol;


#[derive(Clone, Debug, Default)]
pub struct CommandMsg {
    pub start: bool, 
    pub port: u16,
    // pub protocol: String,
    pub protocol: Protocol,
    pub bind_ip: String,
    pub path: String,
}

impl CommandMsg {
    pub fn new(prot: &Protocol) -> Self {
        Self {
            start: false, 
            port: prot.get_default_port(),
            protocol: prot.clone(),
            ..Default::default()
        }
    }
}
