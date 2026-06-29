#![allow(
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::module_name_repetitions
)]
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::collections::HashSet;

// ── Width strategy ──────────────────────────────────────────────────────

/// How a column's width is determined.
#[derive(Debug, Clone)]
pub enum ColumnWidth {
    /// Exact pixel width.
    Fixed(u16),
    /// Proportional weight for distributing remaining space (e.g. `Ratio(3)`).
    Ratio(u16),
    /// Content-based with a minimum pixel width.
    Min(u16),
}

// ── Sort ────────────────────────────────────────────────────────────────

/// Sort direction indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

// ── Column ──────────────────────────────────────────────────────────────

/// A single column definition.
#[derive(Debug, Clone)]
pub struct Column<'a> {
    pub header: Line<'a>,
    pub width: ColumnWidth,
    pub alignment: Alignment,
    pub style: Style,
}

impl<'a> Column<'a> {
    #[must_use]
    pub fn new(header: impl Into<Line<'a>>, width: ColumnWidth) -> Self {
        Self {
            header: header.into(),
            width,
            alignment: Alignment::Left,
            style: Style::default(),
        }
    }

    #[must_use]
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

// ── Row trait ───────────────────────────────────────────────────────────

/// A row that can be rendered into pre-computed column slots.
pub trait DataTableRow {
    /// Render row content into the buffer at the given y position.
    /// `col_xs` gives the starting x coordinate of each column.
    /// `col_widths` gives the pixel width of each column.
    fn render(&self, col_xs: &[u16], col_widths: &[u16], buf: &mut Buffer, y: u16);

    /// Row height in lines (default 1).
    /// Override for wrapping/multi-line rows such as log messages.
    fn height(&self, _available_width: u16) -> u16 {
        1
    }
}

// ── Widget ──────────────────────────────────────────────────────────────

/// A stateful data table widget similar to a spreadsheet or list view.
///
/// Columns are defined with a width strategy. Rows implement [`DataTableRow`]
/// to control per-cell rendering. Supports single-selection, multi-selection,
/// column sorting indicators, a header row with optional divider, and an
/// optional block border.
pub struct DataTable<'a, R: DataTableRow> {
    pub columns: Vec<Column<'a>>,
    pub rows: &'a [R],
    pub highlight_style: Style,
    pub selection_style: Style,
    pub sort_column: Option<usize>,
    pub sort_direction: SortDirection,
    pub column_spacing: u16,
    pub block: Option<Block<'a>>,
    pub header_divider: Option<char>,
}

impl<R: DataTableRow> Default for DataTable<'_, R> {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: &[],
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
            selection_style: Style::default(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            column_spacing: 1,
            block: None,
            header_divider: None,
        }
    }
}

impl<'a, R: DataTableRow> DataTable<'a, R> {
    #[must_use]
    pub fn new(columns: Vec<Column<'a>>, rows: &'a [R]) -> Self {
        Self {
            columns,
            rows,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    #[must_use]
    pub fn selection_style(mut self, style: Style) -> Self {
        self.selection_style = style;
        self
    }

    #[must_use]
    pub fn sort_column(mut self, col: Option<usize>, dir: SortDirection) -> Self {
        self.sort_column = col;
        self.sort_direction = dir;
        self
    }

    #[must_use]
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use]
    pub fn header_divider(mut self, ch: char) -> Self {
        self.header_divider = Some(ch);
        self
    }
}

// ── State ───────────────────────────────────────────────────────────────

/// Mutable state for [`DataTable`].
#[derive(Debug, Clone)]
pub struct DataTableState {
    /// Scroll offset (index of first visible row).
    pub offset: usize,
    /// Currently selected row index (global).
    pub selected: Option<usize>,
    /// Multi-selected row indices (global).
    pub multi_selected: HashSet<usize>,
}

impl Default for DataTableState {
    fn default() -> Self {
        Self {
            offset: 0,
            selected: Some(0),
            multi_selected: HashSet::new(),
        }
    }
}

// ── Width computation ───────────────────────────────────────────────────

/// Compute pixel widths for each column given the available inner width.
fn compute_widths(columns: &[Column], available: u16, spacing: u16) -> Vec<u16> {
    if columns.is_empty() {
        return Vec::new();
    }

    let total_spacing = columns.len().saturating_sub(1) as u16 * spacing;
    let available = available.saturating_sub(total_spacing);

    let mut widths: Vec<u16> = vec![0; columns.len()];
    let mut remaining = available;

    // 1. Assign Fixed widths
    for (i, col) in columns.iter().enumerate() {
        if let ColumnWidth::Fixed(w) = col.width {
            let w = w.min(remaining);
            widths[i] = w;
            remaining = remaining.saturating_sub(w);
        }
    }

    // 2. Assign Min widths
    for (i, col) in columns.iter().enumerate() {
        if let ColumnWidth::Min(min) = col.width {
            let w = min.min(remaining);
            widths[i] = w;
            remaining = remaining.saturating_sub(w);
        }
    }

    // 3. Distribute remaining to Ratio columns
    let ratio_total: u16 = columns
        .iter()
        .filter_map(|c| match c.width {
            ColumnWidth::Ratio(r) => Some(r),
            _ => None,
        })
        .sum();

    if ratio_total > 0 && remaining > 0 {
        let remaining_before_ratio = remaining;
        let mut assigned = 0u16;
        for (i, col) in columns.iter().enumerate() {
            if let ColumnWidth::Ratio(r) = col.width {
                let share = remaining_before_ratio * r / ratio_total;
                widths[i] = share;
                assigned += share;
            }
        }
        // Give any leftover pixels to the last ratio column
        if let Some(ratio_idx) = columns
            .iter()
            .enumerate()
            .rev()
            .find(|(_, c)| matches!(c.width, ColumnWidth::Ratio(_)))
            .map(|(i, _)| i)
        {
            widths[ratio_idx] += remaining - assigned;
        }
    }

    widths
}

impl<R: DataTableRow> StatefulWidget for DataTable<'_, R> {
    type State = DataTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = self.block.as_ref().map_or(area, |block| {
            let cloned_block = block.clone();
            cloned_block.render(area, buf);
            block.inner(area)
        });

        if inner.width == 0 || inner.height == 0 {
            return;
        }
        if self.columns.is_empty() || self.rows.is_empty() {
            return;
        }

        // Compute column pixel widths
        let col_widths = compute_widths(&self.columns, inner.width, self.column_spacing);
        let col_xs: Vec<u16> = {
            let mut xs = Vec::with_capacity(col_widths.len());
            let mut x = inner.x;
            for (i, w) in col_widths.iter().enumerate() {
                xs.push(x);
                x += w;
                if i < col_widths.len() - 1 {
                    x += self.column_spacing;
                }
            }
            xs
        };

        // Header row
        let mut header_y = inner.y;
        for (i, col) in self.columns.iter().enumerate() {
            let w = col_widths[i];
            if w == 0 {
                continue;
            }
            let header_text = if self.sort_column == Some(i) {
                let arrow = match self.sort_direction {
                    SortDirection::Ascending => " ↑",
                    SortDirection::Descending => " ↓",
                };
                // Append a sort-indicator span to the header text
                let mut spans = col.header.spans.clone();
                if let Some(last) = spans.last_mut() {
                    let new_content = format!("{}{}", last.content, arrow);
                    last.content = std::borrow::Cow::Owned(new_content);
                }
                Line::from(spans)
            } else {
                col.header.clone()
            };
            buf.set_line(col_xs[i], header_y, &header_text, w);
        }

        header_y += 1;

        // Divider
        if let Some(ch) = self.header_divider
            && header_y < inner.bottom()
        {
            for x in inner.x..inner.right() {
                buf[(x, header_y)].set_char(ch);
                buf[(x, header_y)].set_style(self.columns[0].style); // use first col style
            }
            header_y += 1;
        }

        // Compute row heights
        let row_heights: Vec<u16> = self
            .rows
            .iter()
            .map(|row| row.height(inner.width))
            .collect();

        // Clamp offset — count rows from the end so max_offset is the
        // earliest visible row that still fills the viewport.
        let total_rows = self.rows.len();
        let visible_height = inner.bottom().saturating_sub(header_y);
        let mut rows_from_end = 0usize;
        let mut h_sum = 0u16;
        for h in row_heights.iter().rev() {
            if h_sum + h > visible_height {
                break;
            }
            h_sum += h;
            rows_from_end += 1;
        }
        let max_offset = total_rows.saturating_sub(rows_from_end);
        if state.offset > max_offset {
            state.offset = max_offset;
        }

        // Render visible rows
        let mut y = header_y;
        let mut row_idx = state.offset;
        while y < inner.bottom() && row_idx < total_rows {
            let rh = row_heights[row_idx];
            if rh == 0 {
                row_idx += 1;
                continue;
            }
            if y + rh > inner.bottom() {
                break;
            }

            let is_selected = state.selected == Some(row_idx);
            let is_multi = state.multi_selected.contains(&row_idx);

            // Apply selection/highlight styles via buffer background
            if is_selected {
                for row_y in y..y + rh {
                    for x in inner.x..inner.right() {
                        buf[(x, row_y)].set_style(self.highlight_style);
                    }
                }
            } else if is_multi {
                for row_y in y..y + rh {
                    for x in inner.x..inner.right() {
                        buf[(x, row_y)].set_style(self.selection_style);
                    }
                }
            }

            self.rows[row_idx].render(&col_xs, &col_widths, buf, y);

            y += rh;
            row_idx += 1;
        }

        // Update selected within bounds
        if let Some(sel) = state.selected {
            if sel >= total_rows {
                state.selected = if total_rows > 0 { Some(0) } else { None };
            }
        } else if total_rows > 0 {
            state.selected = Some(0);
        }
    }
}

impl<R: DataTableRow> Widget for DataTable<'_, R> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = DataTableState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}
