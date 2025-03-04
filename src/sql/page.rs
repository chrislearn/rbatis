use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

use futures_core::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// default 10
pub const DEFAULT_PAGE_SIZE: u64 = 10;

///Page interface, support get_pages() and offset()
pub trait IPageRequest: Send + Sync {
    fn get_page_size(&self) -> u64;
    fn get_page_no(&self) -> u64;
    fn get_total(&self) -> u64;
    fn is_search_count(&self) -> bool;

    fn set_total(self, arg: u64) -> Self;
    fn set_page_size(self, arg: u64) -> Self;
    fn set_page_no(self, arg: u64) -> Self;
    fn set_search_count(self, arg: bool) -> Self;

    ///sum pages
    fn get_pages(&self) -> u64 {
        if self.get_page_size() == 0 {
            return 0;
        }
        let mut pages = self.get_total() / self.get_page_size();
        if self.get_total() % self.get_page_size() != 0 {
            pages = pages + 1;
        }
        return pages;
    }
    ///sum offset
    fn offset(&self) -> u64 {
        if self.get_page_no() > 0 {
            (self.get_page_no() - 1) * self.get_page_size()
        } else {
            0
        }
    }
}

///Page interface, support get_pages() and offset()
pub trait IPage<T>: IPageRequest {
    fn get_records(&self) -> &Vec<T>;
    fn get_records_mut(&mut self) -> &mut Vec<T>;
    fn set_records(self, arg: Vec<T>) -> Self;
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Page<T> {
    /// data
    pub records: Vec<T>,
    /// total num
    pub total: u64,
    /// pages
    pub pages: u64,
    /// current page index
    pub page_no: u64,
    /// default 10
    pub page_size: u64,
    /// is search_count
    pub search_count: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct PageRequest {
    /// total num
    pub total: u64,
    /// current page index
    pub page_no: u64,
    /// page page_size default 10
    pub page_size: u64,
    pub search_count: bool,
}

impl PageRequest {
    pub fn new(page_no: u64, page_size: u64) -> Self {
        return PageRequest::new_total(page_no, page_size, DEFAULT_PAGE_SIZE);
    }

    pub fn new_option(page_no: &Option<u64>, page_size: &Option<u64>) -> Self {
        return PageRequest::new(page_no.unwrap_or(1), page_size.unwrap_or(DEFAULT_PAGE_SIZE));
    }

    pub fn new_total(page_no: u64, page_size: u64, total: u64) -> Self {
        return PageRequest::new_plugin(String::new(), page_no, page_size, total);
    }

    pub fn new_plugin(plugin: String, page_no: u64, page_size: u64, total: u64) -> Self {
        let mut page_no = page_no;
        if page_no < 1 {
            page_no = 1;
        }
        return Self {
            total,
            page_size,
            page_no: page_no,
            search_count: true,
        };
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        return PageRequest {
            total: 0,
            page_size: DEFAULT_PAGE_SIZE,
            page_no: 1,
            search_count: true,
        };
    }
}

impl IPageRequest for PageRequest {
    fn get_page_size(&self) -> u64 {
        self.page_size
    }

    fn get_page_no(&self) -> u64 {
        self.page_no
    }

    fn get_total(&self) -> u64 {
        self.total
    }

    fn is_search_count(&self) -> bool {
        self.search_count
    }

    fn set_total(mut self, total: u64) -> Self {
        self.total = total;
        self
    }

    fn set_page_size(mut self, arg: u64) -> Self {
        self.page_size = arg;
        self
    }

    fn set_page_no(mut self, arg: u64) -> Self {
        self.page_no = arg;
        self
    }

    fn set_search_count(mut self, arg: bool) -> Self {
        self.search_count = arg;
        self
    }
}

impl<T> Page<T> {
    pub fn new(current: u64, page_size: u64) -> Self {
        return Page::new_total(current, page_size, 0);
    }

    pub fn new_option(current: &Option<u64>, page_size: &Option<u64>) -> Self {
        return Page::new(current.unwrap_or(1), page_size.unwrap_or(DEFAULT_PAGE_SIZE));
    }

    pub fn new_total(page_no: u64, page_size: u64, total: u64) -> Self {
        if page_no < 1 {
            return Self {
                total,
                pages: {
                    let mut pages = total / page_size;
                    if total % page_size != 0 {
                        pages += 1;
                    }
                    pages
                },
                page_size: page_size,
                page_no: 1 as u64,
                records: vec![],
                search_count: true,
            };
        }
        return Self {
            total,
            pages: {
                let mut pages = total / page_size;
                if total % page_size != 0 {
                    pages += 1;
                }
                pages
            },
            page_size: page_size,
            page_no,
            records: vec![],
            search_count: true,
        };
    }
}

impl<T> Default for Page<T> {
    fn default() -> Self {
        return Page {
            records: vec![],
            total: 0,
            pages: 0,
            page_size: DEFAULT_PAGE_SIZE,
            page_no: 1,
            search_count: true,
        };
    }
}

impl<T> IPageRequest for Page<T>
    where
        T: Send + Sync,
{
    fn get_page_size(&self) -> u64 {
        self.page_size
    }
    fn get_page_no(&self) -> u64 {
        self.page_no
    }

    fn get_total(&self) -> u64 {
        self.total
    }

    fn is_search_count(&self) -> bool {
        self.search_count
    }

    fn set_total(mut self, total: u64) -> Self {
        self.total = total;
        self
    }

    fn set_page_size(mut self, arg: u64) -> Self {
        self.page_size = arg;
        self
    }

    fn set_page_no(mut self, arg: u64) -> Self {
        self.page_no = arg;
        self
    }

    fn set_search_count(mut self, arg: bool) -> Self {
        self.search_count = arg;
        self
    }
}

impl<T> IPage<T> for Page<T>
    where
        T: Send + Sync,
{
    fn get_records(&self) -> &Vec<T> {
        self.records.as_ref()
    }

    fn get_records_mut(&mut self) -> &mut Vec<T> {
        self.records.as_mut()
    }

    fn set_records(mut self, arg: Vec<T>) -> Self {
        self.records = arg;
        self
    }
}

impl<T: Display + Debug> Display for Page<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("records", &self.records)
            .field("total", &self.total)
            .field("pages", &self.pages)
            .field("page_no", &self.page_no)
            .field("page_size", &self.page_size)
            .field("search_count", &self.search_count)
            .finish()
    }
}

impl<V> Page<V> {
    pub fn from<T>(arg: Page<T>) -> Self where V: From<T> {
        let mut p = Page::<V>::new(arg.page_no, arg.page_size);
        p.pages = arg.pages;
        p.page_no = arg.page_no;
        p.page_size = arg.page_size;
        p.total = arg.total;
        p.search_count = arg.search_count;
        p.records = {
            let mut records = Vec::with_capacity(arg.records.len());
            for x in arg.records {
                records.push(V::from(x));
            }
            records
        };
        p
    }
}

#[cfg(test)]
mod test {
    use crate::sql::page::Page;

    #[test]
    fn test_page() {
        let mut page = Page::<i32>::new_total(1, 10, 1);
        page.records = vec![];
        println!("{:?}", page);
        assert_eq!(page.pages, 1);
    }
}
