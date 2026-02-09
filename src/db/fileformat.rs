//! Interal module for handling file format operations.
//! It's not part of the public API.
//!
//! It is based on SQlite documentation about file format:
//! [Sqlite fileformat doc](https://www.sqlite.org/fileformat.html)
//!
//! It contains three main modules:
//! * [page] A module that offer a way to read page in the db
//! * [record] A module that allows to read one record
//! * [types]  Type associated of the fileformat

use crate::db::fileformat::{interior_page::InteriorPage, page::Page};
use anyhow::{Result, anyhow};

pub mod interior_page;
pub mod page;
pub mod record;
pub mod types;

pub enum PageType {
    InteriorPage(InteriorPage),
    Page(Page),
}

impl PageType {
    pub fn from_buffer(buffer: Vec<u8>, page_number: usize) -> Result<PageType> {
        match buffer[0] {
            0x05 => Ok(PageType::InteriorPage(InteriorPage::new(
                buffer,
                page_number,
            )?)),
            0x0d => Ok(PageType::Page(Page::new(buffer, page_number)?)),
            _ => Err(anyhow!("Page: unknow page type")),
        }
    }
}
