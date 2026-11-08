//! Pagination support for list endpoints.
//!
//! # Spring Boot Comparison
//!
//! In Spring Boot, you'd use `Pageable` and `Page<T>` from Spring Data:
//! ```java
//! @GetMapping("/accounts")
//! public Page<Account> list(Pageable pageable) { ... }
//! ```
//!
//! In Axum, we define our own pagination types and extract them from query parameters
//! using `Query<PaginationParams>`.

use serde::{Deserialize, Serialize};

/// Query parameters for paginated list endpoints.
///
/// Extracted from URL query string: `?page=2&per_page=20`
///
/// Both fields are optional with sensible defaults.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl PaginationParams {
    /// Current page number (1-indexed, minimum 1).
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    /// Items per page (default 20, max 100, minimum 1).
    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }

    /// SQL OFFSET value computed from page and per_page.
    pub fn offset(&self) -> u32 {
        (self.page() - 1) * self.per_page()
    }
}

/// Pagination metadata included in list responses.
///
/// This is returned in the `meta` field of `ApiResponse` so clients
/// know total counts and can build pagination UI.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            total_items.div_ceil(per_page as u64) as u32
        };
        Self {
            page,
            per_page,
            total_items,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults() {
        let params = PaginationParams {
            page: None,
            per_page: None,
        };
        assert_eq!(params.page(), 1);
        assert_eq!(params.per_page(), 20);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_pagination_page_2() {
        let params = PaginationParams {
            page: Some(2),
            per_page: Some(10),
        };
        assert_eq!(params.offset(), 10);
    }

    #[test]
    fn test_pagination_clamps_per_page() {
        let params = PaginationParams {
            page: Some(1),
            per_page: Some(500),
        };
        assert_eq!(params.per_page(), 100); // Clamped to max
    }

    #[test]
    fn test_pagination_zero_page_becomes_one() {
        let params = PaginationParams {
            page: Some(0),
            per_page: None,
        };
        assert_eq!(params.page(), 1);
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(2, 10, 35);
        assert_eq!(meta.total_pages, 4); // ceil(35/10) = 4
    }

    #[test]
    fn test_pagination_meta_exact_division() {
        let meta = PaginationMeta::new(1, 10, 30);
        assert_eq!(meta.total_pages, 3);
    }
}
