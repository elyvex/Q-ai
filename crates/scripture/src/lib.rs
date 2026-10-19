//! Scripture domain types (Phase 9).

pub mod book;
pub mod edition;
pub mod reference;
pub mod status;
pub mod translation;
pub mod verse;
pub mod view;

pub use book::{ScriptureBook, ScriptureBookId};
pub use edition::{ScriptureEdition, ScriptureEditionId, ScriptureEditionStatistics, Tradition};
pub use reference::ScriptureRef;
pub use status::{
    EditionStatus, RecordStatus, ScriptureEditionStatus, ScriptureStatus, ScriptureVerseStatus,
    VerseStatus,
};
pub use translation::ScriptureTranslation;
pub use verse::{ScriptureError, ScriptureVerse, ScriptureVerseError, ScriptureVerseId};
pub use view::ScriptureView;
