pub mod index;
pub mod model;
pub mod refs;

pub use index::{WorkRecordIndex, WorkRecordMatch, WorkRecordSearchScope};
pub use model::{
    ChatMessage, WorkChannel, WorkOutcome, WorkPayload, WorkRecord, WorkRecordKind, WorkSessionRef,
    WorkSource, WorkStatus, WorkText, WorkTime, RECORD_SCHEMA_VERSION,
};
pub use refs::{WorkRef, WorkRefTarget};
