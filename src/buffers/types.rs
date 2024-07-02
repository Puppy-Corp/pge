#[derive(Debug, Clone)]
pub struct WriteCommand {
    pub offset: usize,
    pub data: Vec<u8>,
}