#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    StartSearch,
    SubmitSearch,
    CancelSearch,
    SearchInput(char),
    SearchBackspace,
    SelectNext,
    SelectPrev,
    PlaySelected,
    TogglePause,
    ToggleLoop,
}
