use crate::domain::types::*;

#[derive(Debug, Clone)]
pub enum Message {
    Digit(char),
    Decimal,
    BinaryOp(BinaryOp),
    UnaryFunc(UnaryFunc),
    PostfixOp(PostfixOp),
    Constant(f64, &'static str),
    Equals,
    Clear,
    Backspace,
    ToggleSign,
    LeftParen,
    RightParen,
    EE,

    MemoryClear,
    MemoryRecall,
    MemoryAdd,
    MemorySubtract,
    MemoryStore,

    ToggleAngleMode,
    Undo,

    NewTab,
    CloseTab,
    SwitchTab(usize),
    NextTab,
    PrevTab,
    RenameTab(usize, String),

    ToggleScientific,
    ToggleTheme,

    ToggleHistory,
    ToggleMemory,
    TogglePinned,
    PinResult,
    SearchHistory(String),
    ClearHistory,
    ExportHistoryJson,
    ExportHistoryCsv,

    OpenConverter,
    OpenTools,
    OpenNotes,
    CloseMode,
    ShowHelp,
    Quit,

    Navigate(crate::ui::keyboard::Direction),
    Activate,
    OpenMenu,

    Noop,
}
