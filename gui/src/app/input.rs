#[derive(Debug)]
pub enum AppInput {
    UpdateFont(String),
    UpdateFontSize(f32),
    UpdateTextColor(String),
    UpdateBgColor(String),
    UpdateBgOpacity(f32),
    UpdateBgEnabled(bool),
    UpdateStrokeColor(String),
    UpdateStrokeEnabled(bool),
    UpdateStrokeWidth(f32),
    UpdateShadowColor(String),
    UpdateShadowEnabled(bool),
    UpdateShadowOffset(f32),
    UpdateAuthorWeight(String, u32),
    Save,
    ShowInteractivePicker,
    FetchQuoteNow,
}
