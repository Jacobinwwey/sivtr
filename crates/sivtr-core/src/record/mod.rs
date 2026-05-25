pub mod index;
pub mod model;
pub mod refs;

pub use index::WorkRecordIndex;
pub use model::{
    chat_turn_ranges, is_real_user_block, ChatMessage, RecordText, RecordTextMode, WorkOutcome,
    WorkPayload, WorkRecord, WorkRecordCopyParts, WorkRecordKind, WorkStatus, WorkText, WorkTime,
    RECORD_SCHEMA_VERSION,
};
pub use refs::{WorkRef, WorkRefSelector, WorkRefTarget};
