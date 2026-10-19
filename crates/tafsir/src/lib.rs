//! Tafsir domain types (Phase 6).
//!
//! # tafsir
//!
//! Source-aware tafsir records following the Quran corpus pattern. Per D-06
//! (locked): tafsir is indexed by work/author/dimensions following the
//! existing catalog pattern. Each record carries its work name, author,
//! dimensions (volume, page, chapter), and linked verses (via `verse_links`,
//! plan 06-03).

pub mod record;
pub mod work;

pub use record::{TafsirId, TafsirRecord};
pub use work::{TafsirDimensions, TafsirWork, TafsirWorkId, WorkStatus};
