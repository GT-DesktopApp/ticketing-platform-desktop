// Generic, reusable pagination types shared by every list endpoint.
//
// The frontend sends page + per_page (driven by the "Rows: 25" selector); the
// backend clamps per_page so a client can't request an unbounded page, and
// returns the total so the UI can render page counts and the |< < > >| controls.

use serde::{Deserialize, Serialize};

/// Hard ceiling on rows per page, enforced server-side regardless of request.
pub const MAX_PER_PAGE: u32 = 100;
/// Default page size when the client omits per_page.
pub const DEFAULT_PER_PAGE: u32 = 25;

/// A validated pagination + filter request.
#[derive(Debug, Clone)]
pub struct PageRequest {
    pub page: u32,      // 1-based
    pub per_page: u32,  // already clamped to [1, MAX_PER_PAGE]
    pub search: Option<String>,
    pub active_only: bool,
}

impl PageRequest {
    /// Build a sanitised request from raw command args.
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        active_only: Option<bool>,
    ) -> Self {
        let page = page.unwrap_or(1).max(1);
        let per_page = per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE);
        let search = search
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        PageRequest {
            page,
            per_page,
            search,
            active_only: active_only.unwrap_or(false),
        }
    }

    /// SQL OFFSET for the current page.
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.per_page) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }
}

/// A page of results plus the metadata the UI needs.
#[derive(Debug, Serialize, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

impl<T> Page<T> {
    pub fn new(data: Vec<T>, req: &PageRequest, total: i64) -> Self {
        let per_page = req.per_page.max(1);
        let total_pages = if total <= 0 {
            0
        } else {
            ((total as u32) + per_page - 1) / per_page
        };
        Page {
            data,
            page: req.page,
            per_page,
            total,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_per_page_to_max() {
        let r = PageRequest::new(Some(1), Some(5000), None, None);
        assert_eq!(r.per_page, MAX_PER_PAGE);
    }

    #[test]
    fn defaults_apply() {
        let r = PageRequest::new(None, None, None, None);
        assert_eq!(r.page, 1);
        assert_eq!(r.per_page, DEFAULT_PER_PAGE);
        assert!(!r.active_only);
        assert!(r.search.is_none());
    }

    #[test]
    fn page_zero_becomes_one() {
        let r = PageRequest::new(Some(0), None, None, None);
        assert_eq!(r.page, 1);
        assert_eq!(r.offset(), 0);
    }

    #[test]
    fn offset_computed_from_page() {
        let r = PageRequest::new(Some(3), Some(25), None, None);
        assert_eq!(r.offset(), 50); // (3-1)*25
    }

    #[test]
    fn blank_search_is_none() {
        let r = PageRequest::new(None, None, Some("   ".into()), None);
        assert!(r.search.is_none());
    }

    #[test]
    fn total_pages_rounds_up() {
        let req = PageRequest::new(Some(1), Some(25), None, None);
        let page = Page::new(Vec::<i32>::new(), &req, 51);
        assert_eq!(page.total_pages, 3); // ceil(51/25)
    }

    #[test]
    fn total_pages_zero_when_empty() {
        let req = PageRequest::new(Some(1), Some(25), None, None);
        let page = Page::new(Vec::<i32>::new(), &req, 0);
        assert_eq!(page.total_pages, 0);
    }
}
