use super::PageInfo;

pub enum FreedOrPageInfo {
    Freed(usize),
    PageInfo(PageInfo),
}
