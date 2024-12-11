use std::collections::HashMap;

#[derive(Clone)]

pub struct status_ms {
    pub status_color: HashMap<u16, String>,
    pub status_message: HashMap<u16, String>,
    pub suggestion_fix_message: HashMap<u16, HashMap<u16, String>>,
}


pub struct Handler {

}