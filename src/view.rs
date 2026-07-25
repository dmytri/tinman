//! The capture view: a Ratatui widget that renders a virtual screen.
//!
//! The operator watches the captured program while recording, so the view draws
//! each cell of the virtual screen into the frame at its own position.

use crate::screen::VirtualScreen;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

/// A Ratatui widget that renders a virtual screen's rows into the frame.
///
/// @planks("the capture view is rendered to a {int} by {int} test terminal")
pub struct CaptureView<'a> {
    screen: &'a VirtualScreen,
}

impl<'a> CaptureView<'a> {
    /// Build a capture view over a virtual screen.
    ///
    /// @planks("the capture view is rendered to a {int} by {int} test terminal")
    pub fn new(screen: &'a VirtualScreen) -> Self {
        CaptureView { screen }
    }
}

impl Widget for CaptureView<'_> {
    /// @planks("the capture view is rendered to a {int} by {int} test terminal")
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.screen.rows();
        for y in 0..area.height {
            let Some(row) = rows.get(y as usize) else {
                break;
            };
            for x in 0..area.width {
                let Some(symbol) = row.get(x as usize) else {
                    break;
                };
                if symbol.is_empty() {
                    continue;
                }
                if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                    cell.set_symbol(symbol);
                }
            }
        }
    }
}
