use crate::servers::server::Protocol;


#[derive(Clone, Debug, Default)]
pub struct CommandMsg {
    pub toggle: bool, 
    pub port: u16,
    pub name: String,
    pub bind_ip: String,
    pub path: String,
}

impl CommandMsg {
    pub fn new(prot: &Protocol) -> Self {
        Self {
            toggle: false, 
            port: prot.get_default_port(),
            name: prot.to_string().into(),
            ..Default::default()
        }
    }
}
