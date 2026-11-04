mod fragments;
use crate::Outcome;
use biodata::Text;
use biomcp_adapters::sheet;
use biomcp_fetch::LocalDeadline;
use biomcp_types::{
    ArticleSupplement, PageInfo, Reason, Request, SheetInfo, SheetRow, StepKind, SupplementContent,
};
const PAGE_BYTES: usize = 4000;
struct Page {
    offset: u32,
    total: u32,
    bytes: usize,
    rows: Vec<SheetRow>,
    full: bool,
    end: LocalDeadline,
}
impl Page {
    fn push(&mut self, row: SheetRow) -> Result<(), Reason> {
        if self.end.expired() {
            return Err(Reason::DeadlineExceeded);
        }
        let position = self.total;
        self.total = self.total.checked_add(1).ok_or(Reason::TooLarge)?;
        if position < self.offset || self.full {
            return Ok(());
        }
        let encoded = serde_json::to_vec(&row).map_err(|_| Reason::BadReply)?;
        if self.rows.len() == 25 || !self.rows.is_empty() && self.bytes + encoded.len() > PAGE_BYTES
        {
            self.full = true;
            return Ok(());
        }
        self.bytes += encoded.len();
        self.rows.push(row);
        Ok(())
    }
}
pub(super) fn select(
    query: &ArticleSupplement,
    sheets: &[sheet::Sheet],
    content: &mut SupplementContent,
    out: &mut Outcome,
    selected: &[String],
    end: LocalDeadline,
) -> Result<bool, Reason> {
    if query.rows().is_none() && selected.is_empty() {
        overview(query, sheets, content, out);
        return Ok(true);
    }
    if query.rows().is_some()
        && (sheets.len() > 1 && query.sheet().is_none()
            || query
                .sheet()
                .is_some_and(|name| !sheets.iter().any(|s| s.name == name.as_str())))
    {
        return Ok(false);
    }
    let needles: Vec<_> = selected.iter().map(|v| v.to_lowercase()).collect();
    let mut page = Page {
        offset: query.offset(),
        total: 0,
        bytes: 0,
        rows: Vec::new(),
        full: false,
        end,
    };
    if needles.is_empty() {
        collect(query, sheets, &needles, None, &mut page)?;
    } else {
        collect(query, sheets, &needles, Some(true), &mut page)?;
        // A search page spends its budget on matches before returning context.
        // The remaining fragments keep their source coordinates and page offsets.
        if !page.rows.is_empty() {
            page.full = true;
        }
        collect(query, sheets, &needles, Some(false), &mut page)?;
        if !page.rows.iter().any(|r| fragments::matches(r, &needles)) {
            content.search_message = Some(
                if page.total == 0 {
                    "No row contains the search term."
                } else {
                    "No row fragment on this page contains the search term."
                }
                .into(),
            );
        }
    }
    if query.offset() > page.total {
        return Ok(false);
    }
    let returned = u32::try_from(page.rows.len()).map_err(|_| Reason::TooLarge)?;
    let next = query
        .offset()
        .checked_add(returned)
        .ok_or(Reason::TooLarge)?;
    if next < page.total && returned > 0 {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &super::super::next::text_sources(query.input()),
            Request::ArticleSupplement(query.page(next)),
            false,
        ));
    }
    out.page = Some(PageInfo {
        offset: query.offset(),
        returned,
        total: u64::from(page.total),
        pool: None,
        truncated: None,
    });
    if !needles.is_empty() {
        context_steps(query, sheets, &page.rows, out)?;
    }
    content.rows = Some(page.rows.into());
    Ok(true)
}
fn context_steps(
    query: &ArticleSupplement,
    sheets: &[sheet::Sheet],
    rows: &[SheetRow],
    out: &mut Outcome,
) -> Result<(), Reason> {
    for sheet in sheets
        .iter()
        .filter(|sheet| rows.iter().any(|row| row.sheet.as_str() == sheet.name))
    {
        out.next.push(crate::next::Step::run(
            StepKind::Section,
            &super::super::next::text_sources(query.input()),
            Request::ArticleSupplement(query.first_rows(
                Text::new(&sheet.name).map_err(|_| Reason::BadReply)?,
                sheet.rows.last().map_or(1, |row| row.number),
            )),
            false,
        ));
    }
    Ok(())
}
fn collect(
    query: &ArticleSupplement,
    sheets: &[sheet::Sheet],
    needles: &[String],
    matching: Option<bool>,
    page: &mut Page,
) -> Result<(), Reason> {
    for sheet in sheets {
        if query.sheet().is_some_and(|v| v.as_str() != sheet.name) {
            continue;
        }
        let empty_first_row = sheet::Row {
            number: 1,
            cells: Vec::new(),
        };
        let first_row = sheet
            .rows
            .first()
            .filter(|row| row.number == 1)
            .unwrap_or(&empty_first_row);
        let name = Text::new(&sheet.name).map_err(|_| Reason::BadReply)?;
        for row in &sheet.rows {
            if page.end.expired() {
                return Err(Reason::DeadlineExceeded);
            }
            let included = if let Some(range) = query.rows() {
                row.number >= range.start() && row.number <= range.end()
            } else {
                row.cells.iter().any(|v| fragments::contains(v, needles))
            };
            if included {
                fragments::push(page, name.clone(), row, first_row, needles, matching)?;
            }
        }
    }
    Ok(())
}

fn overview(
    query: &ArticleSupplement,
    sheets: &[sheet::Sheet],
    content: &mut SupplementContent,
    out: &mut Outcome,
) {
    content.sheets = Some(
        sheets
            .iter()
            .filter_map(|s| {
                Some(SheetInfo {
                    name: Text::new(&s.name).ok()?,
                    rows: s.rows.last().map_or(0, |r| r.number),
                })
            })
            .collect(),
    );
    if let Some(first) = sheets.first()
        && let Some(name) = crate::next::checked(
            Text::new(&first.name),
            biomcp_types::Source::PmcOa,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &super::super::next::text_sources(query.input()),
            Request::ArticleSupplement(
                query.first_rows(name, first.rows.last().map_or(1, |r| r.number)),
            ),
            false,
        ));
    }
}
