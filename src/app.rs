use crate::AppResult;
use crate::system_info::SystemInfo;

#[derive(Debug)]
pub struct App {
    pub system_info: SystemInfo,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> AppResult<Self> {
        let system_info = SystemInfo::collect()?;

        Ok(Self {
            system_info,
            should_quit: false,
        })
    }
}
