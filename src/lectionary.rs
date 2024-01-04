use crate::{
    build_error_page, markdown,
    prelude::*,
    templates::{self, PageTemplate},
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};
use html::{content::Heading2, forms::Button, tables::Table};
use std::path::PathBuf;
use thiserror::Error;
use time::{
    macros::{date, format_description},
    Date, Month, OffsetDateTime,
};
use tokio::task::spawn_blocking;

pub type R<T> = core::result::Result<T, Error>;
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Template(#[from] templates::Error),
    #[error(transparent)]
    Markdown(#[from] markdown::Error),

    #[error("Failed to calculate date of Easter")]
    Easter(#[from] time::error::ComponentRange),
}

#[derive(Debug)]
struct LecEntry {
    date: Option<Date>,
    morning: [&'static str; 3],
    evening: [&'static str; 3],
    dscr: Option<&'static str>,
}

pub async fn lectionary(state: State<AppState>) -> Response {
    let root = state.root.clone();
    lectionary_wrapped(state)
        .await
        .unwrap_or_else(|err| build_error_page(root, err.into()))
}

async fn lectionary_wrapped(state: State<AppState>) -> R<Response> {
    let year = OffsetDateTime::now_utc().year();

    let lec = lec(year)?;
    let lec_table = Table::builder()
        .table_head(|th| {
            th.table_row(|tr| {
                tr.table_header(|thdr| thdr.text(format!("Date ({})", year)));
                tr.table_header(|thdr| thdr.text("Morning").colspan("3"));
                tr.table_header(|thdr| thdr.text("Evening").colspan("3"))
            })
        })
        .table_body(|tb| {
            for le in lec.iter() {
                let dscr = match le.dscr {
                    Some(d) => format!("<br>{}", d),
                    None => String::new(),
                };
                let date = match le.date {
                    Some(d) => {
                        let f = format_description!("[day padding:none] [month repr:short]");
                        format!("{}{dscr}", d.format(&f).expect("format error"))
                    }
                    None => format!("No date{dscr}"),
                };
                tb.table_row(|tr| {
                    tr.table_cell(|tc| tc.text(date).class("right-border"));
                    tr.table_cell(|tc| tc.text(le.morning[0]));
                    tr.table_cell(|tc| tc.text(le.morning[1]));
                    tr.table_cell(|tc| tc.text(le.morning[2]).class("right-border"));
                    tr.table_cell(|tc| tc.text(le.evening[0]));
                    tr.table_cell(|tc| tc.text(le.evening[1]));
                    tr.table_cell(|tc| tc.text(le.evening[2]))
                });
            }
            tb
        })
        .class("lectionary")
        .build()
        .to_string();

    let collapse_button = Button::builder()
        .type_("button")
        .class("collapsible")
        .text(
            Heading2::builder()
                .text("Full Lectionary")
                .build()
                .to_string(),
        )
        .build()
        .to_string();

    let today = OffsetDateTime::now_utc().date();
    let lec_today = Table::builder()
        .table_head(|th| {
            th.table_row(|tr| {
                tr.table_header(|thdr| thdr.text(format!("Date ({})", year)));
                tr.table_header(|thdr| thdr.text("Morning").colspan("3"));
                tr.table_header(|thdr| thdr.text("Evening").colspan("3"))
            })
        })
        .table_body(|tb| {
            tb.table_row(|tr| {
                tr.table_cell(|tc| {
                    let f = format_description!("[day padding:none] [month repr:short]");
                    tc.text(format!(
                        "Today's Reading ({})",
                        today.format(&f).expect("format error")
                    ))
                    .class("right-border")
                });
                tr.table_cell(|tc| tc.text(lec[(today.ordinal() - 1) as usize].morning[0]));
                tr.table_cell(|tc| tc.text(lec[(today.ordinal() - 1) as usize].morning[1]));
                tr.table_cell(|tc| {
                    tc.text(lec[(today.ordinal() - 1) as usize].morning[2])
                        .class("right-border")
                });
                tr.table_cell(|tc| tc.text(lec[(today.ordinal() - 1) as usize].evening[0]));
                tr.table_cell(|tc| tc.text(lec[(today.ordinal() - 1) as usize].evening[1]));
                tr.table_cell(|tc| tc.text(lec[(today.ordinal() - 1) as usize].evening[2]))
            })
        })
        .build()
        .to_string();

    let (page, md) = markdown::get_markdown_contents(&state, PathBuf::from("lectionary.md"))?;
    let (pre, post) = md.split_once("</h1>").expect("TODO");
    // let content = format!(r#"{pre}</h1>{lec_today}{post}{collapse_button}{lec_table}"#,);
    let content = format!(r#"{pre}</h1>{lec_today}{post}<h2>Full Lectionary</h2>{lec_table}"#,);

    Ok(Html(
        page.build(&state.root, content)?
            .render()
            .map_err(templates::Error::Template)?,
    )
    .into_response())
}

fn lec(year: i32) -> R<Vec<LecEntry>> {
    let mut lec = lec_skeleton(year);
    let easter: usize = (easter(year)?.ordinal() - 1) as usize; // -1 to make 0 based

    // Insert special days
    // TODO: Fix when the days are
    // Go backwards to get indices right?

    lec.insert(
        easter - 46,
        LecEntry {
            date: None,
            morning: EASTER[0][0],
            evening: EASTER[0][1],
            dscr: Some("Ash Wednesday"),
        },
    );

    lec.insert(
        easter - 3,
        LecEntry {
            date: None,
            morning: EASTER[1][0],
            evening: EASTER[1][1],
            dscr: Some("Maundy Thursday"),
        },
    );

    lec.insert(
        easter - 2,
        LecEntry {
            date: None,
            morning: EASTER[2][0],
            evening: EASTER[2][1],
            dscr: Some("Good Friday"),
        },
    );

    lec.insert(
        easter - 1,
        LecEntry {
            date: None,
            morning: EASTER[3][0],
            evening: EASTER[3][1],
            dscr: Some("Holy Saturday"),
        },
    );

    lec.insert(
        easter,
        LecEntry {
            date: None,
            morning: EASTER[4][0],
            evening: EASTER[4][1],
            dscr: Some("Easter Sunday"),
        },
    );

    lec.insert(
        easter + 39,
        LecEntry {
            date: None,
            morning: EASTER[5][0],
            evening: EASTER[5][1],
            dscr: Some("Ascension Day"),
        },
    );

    // Add dates
    let mut date = Date::from_calendar_date(year, Month::January, 1)?;
    for e in lec.iter_mut() {
        e.date = Some(date);
        date = date
            .next_day()
            .unwrap_or(Date::from_calendar_date(year, Month::January, 1)?);
    }

    Ok(lec)
}

fn lec_skeleton(year: i32) -> Vec<LecEntry> {
    let mut l = MORNING
        .iter()
        .zip(EVENING.iter())
        .enumerate()
        .map(|(i, (m, e))| {
            let psalm_idx = i % 30;
            let morning = [PSALM_MORNING[psalm_idx], m[0], m[1]];
            let evening = [PSALM_EVENING[psalm_idx], e[0], e[1]];

            LecEntry {
                date: None,
                morning,
                evening,
                dscr: None,
            }
        })
        .collect::<Vec<_>>();
    if !time::util::is_leap_year(year) {
        l.remove(58);
    }
    l
}

fn easter(year: i32) -> R<Date> {
    let aa = year % 19;
    let bb = year / 100;
    let cc = year % 100;
    let dd = bb / 4;
    let ee = bb % 4;
    let ff = (bb + 8) / 25;
    let gg = (bb - ff + 1) / 3;
    let hh = (19 * aa + bb - dd - gg + 15) % 30;
    let ii = cc / 4;
    let kk = cc % 4;
    let ll = (32 + 2 * ee + 2 * ii - hh - kk) % 7;
    let mm = (aa + 11 * hh + 22 * ll) / 451;
    let month = ((hh + ll - 7 * mm + 114) / 31) as u8;
    let day = (hh + ll - 7 * mm + 114) % 31 + 1;
    Date::from_calendar_date(year, month.try_into()?, day as u8).map_err(Error::Easter)
}

// ---------------
//      DATA
// ---------------
static EASTER: [[[&'static str; 3]; 2]; 6] = [
    [
        ["Ps. 38", "Isa. 58:1-12", "Luke 18:9-14"],
        ["Ps. 6, 32", "Jonah 3", "1 Cor. 9:24-27"],
    ],
    [
        ["Ps. 41", "Dan. 9", "John 13:1-20"],
        ["Ps. 142 - 143", "Jer. 31", "John 13:21-38"],
    ],
    [
        ["Ps. 40", "Gen. 22:1-36", "Luke 23:18-49"],
        ["Ps. 102", "Isa. 53", "1 Pet. 2"],
    ],
    [
        ["Ps. 88", "Zech. 9", "Luke 23:50-56"],
        ["Ps. 91", "Ex. 13", "Heb. 4"],
    ],
    [
        ["Ps. 118", "Ex. 14", "Luke 24:1-49"],
        ["Ps. 113 - 114", "Ex. 15", "Rom. 6"],
    ],
    [
        ["Ps. 8, 47", "2 Kings 2", "Luke 24:44-53"],
        ["Ps. 21, 24", "Heb 8", "Eph. 4:1-17"],
    ],
];

static PSALM_MORNING: [&'static str; 30] = [
    "Ps. 1 - 5",
    "Ps. 9 - 11",
    "Ps. 15 - 17",
    "Ps. 19 - 21",
    "Ps. 24 - 26",
    "Ps. 30 - 31",
    "Ps. 35 - 36",
    "Ps. 38 - 40",
    "Ps. 44 - 46",
    "Ps. 50 - 52",
    "Ps. 56 - 58",
    "Ps. 62 - 64",
    "Ps. 68",
    "Ps. 71 - 72",
    "Ps. 75 - 77",
    "Ps. 79 - 81",
    "Ps. 86 - 88",
    "Ps. 90 - 92",
    "Ps. 95 - 97",
    "Ps. 102 - 103",
    "Ps. 105",
    "Ps. 107",
    "Ps. 110 - 113",
    "Ps. 116 - 118",
    "Ps. 119:33-72",
    "Ps. 119:105-144",
    "Ps. 120 - 125",
    "Ps. 132 - 135",
    "Ps. 139 - 141",
    "Ps. 144 - 146",
];

static PSALM_EVENING: [&'static str; 30] = [
    "Ps. 6 - 8",
    "Ps. 12 - 14",
    "Ps. 18",
    "Ps. 22 - 23",
    "Ps. 27 - 29",
    "Ps. 32 - 34",
    "Ps. 37",
    "Ps. 41 - 43",
    "Ps. 47 - 49",
    "Ps. 53 - 55",
    "Ps. 59 - 61",
    "Ps. 65 - 67",
    "Ps. 69 - 70",
    "Ps. 73 - 74",
    "Ps. 78",
    "Ps. 82 - 85",
    "Ps. 89",
    "Ps. 93 - 94",
    "Ps. 98 - 101",
    "Ps. 104",
    "Ps. 106",
    "Ps. 108 - 109",
    "Ps. 114 - 115",
    "Ps. 119:1-32",
    "Ps. 119:73-104",
    "Ps. 119:145-176",
    "Ps. 126 - 131",
    "Ps. 136 - 138",
    "Ps. 142 - 143",
    "Ps. 147 - 150",
];

static MORNING: [[&'static str; 2]; 360] = [
    ["Gen. 1", "John 1"],
    ["Gen. 3", "John 2"],
    ["Gen. 6 - 7", "John 3"],
    ["Gen. 10 - 11", "John 4"],
    ["Gen. 14 - 15", "John 5"],
    ["Gen. 18", "John 6"],
    ["Gen. 20 - 21", "John 7"],
    ["Gen. 24", "John 8"],
    ["Gen. 27", "John 9"],
    ["Gen. 29:31 - 30:43", "John 10"],
    ["Gen. 32 - 33", "John 11"],
    ["Gen. 36", "John 12"],
    ["Gen. 38", "John 13"],
    ["Gen. 41", "John 14:1 - 16:4a"],
    ["Gen. 43", "John 16:4b-33"],
    ["Gen. 46", "John. 17"],
    ["Gen. 48", "John. 18"],
    ["Gen. 49:28 - 50:26", "John. 19"],
    ["Ex. 3 - 4", "John. 20"],
    ["Ex. 7 - 8", "John. 21"],
    ["Ex. 10 - 11", "1 Tim. 1"],
    ["Ex. 13", "1 Tim. 2 - 3"],
    ["Ex. 15", "1 Tim. 4"],
    ["Ex. 17 - 18", "1 Tim. 5:1 - 6:2a"],
    ["Ex. 21 :1- 22:15", "1 Tim. 6:2b-21"],
    ["Ex. 24 - 25", "2 Tim. 1"],
    ["Ex. 28", "2 Tim. 2"],
    ["Ex. 30 - 31", "2 Tim. 3"],
    ["Ex. 33 - 34", "2 Tim. 4"],
    ["Ex. 37 - 38", "1 Cor. 1"],
    ["Ex. 40", "1 Cor. 2"],
    ["Lev. 3 - 4", "1 Cor. 3"],
    ["Lev. 6", "1 Cor. 4"],
    ["Lev. 8", "1 Cor. 5 - 6"],
    ["Lev. 10", "1 Cor. 7"],
    ["Lev. 13", "1 Cor. 8"],
    ["Lev. 15", "1 Cor. 9"],
    ["Lev. 17 - 18", "1 Cor. 10:1 - 11:1"],
    ["Lev. 20", "1 Cor. 11:2-34"],
    ["Lev. 22", "1 Cor. 12 - 13"],
    ["Lev. 24", "1 Cor. 14"],
    ["Lev. 26", "1 Cor. 15"],
    ["Num. 1", "1 Cor. 16"],
    ["Num. 3", "2 Cor. 1:1 - 2:4"],
    ["Num. 5", "2 Cor. 2:5 - 3:18"],
    ["Num. 7", "2 Cor. 4 - 5:10"],
    ["Num. 9", "2 Cor. 5:11 - 7:1"],
    ["Num. 11", "2 Cor. 7:2 - 8:24"],
    ["Num. 14", "2 Cor. 9"],
    ["Num. 16", "2 Cor. 10"],
    ["Num. 19", "2 Cor. 11"],
    ["Num. 21", "2 Cor. 12"],
    ["Num. 23", "2 Cor. 13"],
    ["Num. 25", "Mark 1"],
    ["Num. 27", "Mark 2"],
    ["Num. 29", "Mark 3"],
    ["Num. 31", "Mark 4"],
    ["Num. 33", "Mark 5"],
    ["2 Kings 2", "Matt. 7"],
    ["Num. 35:9 - 36:13", "Mark 6"],
    ["Deut. 2", "Mark 7"],
    ["Deut. 4", "Mark 8:1 - 9:1"],
    ["Deut. 6", "Mark 9:2-50"],
    ["Deut. 8", "Mark 10"],
    ["Deut. 10", "Mark 11"],
    ["Deut. 12", "Mark 12"],
    ["Deut. 15", "Mark 13"],
    ["Deut. 17", "Mark 14"],
    ["Deut. 19", "Mark 15"],
    ["Deut. 21", "Mark 16"],
    ["Deut. 23", "Acts 1"],
    ["Deut. 25:5 - 26:19", "Acts 2"],
    ["Deut. 28", "Acts 3"],
    ["Deut. 30", "Acts 4"],
    ["Deut. 32", "Acts 5 - 6"],
    ["Deut. 34", "Acts 7"],
    ["Josh. 2", "Acts 8"],
    ["Josh. 4", "Acts 9"],
    ["Josh. 6", "Acts 10"],
    ["Josh. 8", "Acts 11 - 12"],
    ["Josh. 10", "Acts 13"],
    ["Josh. 12", "Acts 14"],
    ["Josh. 14", "Acts 15"],
    ["Josh. 16", "Acts 16"],
    ["Josh. 18", "Acts 17"],
    ["Josh. 20 - 21", "Acts 18"],
    ["Josh. 23 - 24", "Acts 19"],
    ["Judg. 2:1 - 3:6", "Acts 20"],
    ["Judg. 4", "Acts 21:1-36"],
    ["Judg. 6", "Acts 21:37 - 22:29"],
    ["Judg. 8", "Acts 22:30 - 23:35"],
    ["Judg. 10", "Acts 24"],
    ["Judg. 12", "Acts 25 - 26"],
    ["Judg. 14", "Acts 27"],
    ["Judg. 16", "Acts 28"],
    ["Judg. 18", "Heb. 1"],
    ["Judg. 20", "Heb. 2"],
    ["Ruth 1", "Heb. 3"],
    ["1 Sam. 1", "Heb. 4:1 - 13"],
    ["1 Sam. 3", "Heb. 4:14 - 5:10"],
    ["1 Sam. 5", "Heb. 5:11 - 6:20"],
    ["1 Sam. 7:3-17", "Heb. 7"],
    ["1 Sam. 9", "Heb. 8"],
    ["1 Sam. 11", "Heb. 9"],
    ["1 Sam. 13", "Heb. 10"],
    ["1 Sam. 15", "Heb. 11"],
    ["1 Sam. 17", "Heb. 12"],
    ["1 Sam. 19", "Heb. 13"],
    ["1 Sam. 21", "1 John 1:1 - 2:14"],
    ["1 Sam. 23", "1 John 2:15 - 3:10"],
    ["1 Sam. 25", "1 John 3:11-24"],
    ["1 Sam. 27", "1 John 4"],
    ["1 Sam. 29", "1 John 5"],
    ["1 Sam. 31", "2 John, 3 John"],
    ["2 Sam. 2", "Jude"],
    ["2 Sam. 4", "Matt. 1"],
    ["2 Sam. 6", "Matt. 2"],
    ["2 Sam. 8", "Matt. 3"],
    ["2 Sam. 10", "Matt. 4"],
    ["2 Sam. 12", "Matt. 5"],
    ["2 Sam. 14", "Matt. 6"],
    ["2 Sam. 16", "Matt. 7"],
    ["2 Sam. 18", "Matt. 8"],
    ["2 Sam. 20", "Matt. 9"],
    ["2 Sam. 22", "Matt. 10"],
    ["2 Sam. 24", "Matt. 11"],
    ["1 Kings 2", "Matt. 12"],
    ["1 Kings 4", "Matt. 13"],
    ["1 Kings 6", "Matt. 14"],
    ["1 Kings 8", "Matt. 15"],
    ["1 Kings 10", "Matt. 16"],
    ["1 Kings 12", "Matt. 17"],
    ["1 Kings 14", "Matt. 18"],
    ["1 Kings 16:8-34", "Matt. 19"],
    ["1 Kings 18", "Matt. 20"],
    ["1 Kings 20", "Matt. 21"],
    ["1 Kings 22", "Matt. 22"],
    ["2 Kings 2", "Matt. 23"],
    ["2 Kings 4", "Matt. 24"],
    ["2 Kings 6", "Matt. 25"],
    ["2 Kings 8", "Matt. 26"],
    ["2 Kings 10", "Matt. 27"],
    ["2 Kings 12", "Matt. 28"],
    ["2 Kings 14", "1 Cor. 1"],
    ["2 Kings 16", "1 Cor. 2"],
    ["2 Kings 17:6-41", "1 Cor. 3"],
    ["2 Kings 19", "1 Cor. 4"],
    ["2 Kings 21", "1 Cor. 5 - 6"],
    ["2 Kings 23:1-35", "1 Cor. 7"],
    ["2 Kings 25", "1 Cor. 8"],
    ["1 Chr. 2", "1 Cor. 9"],
    ["1 Chr. 4", "1 Cor. 10:1 - 11:1"],
    ["1 Chr. 6", "1 Cor. 11:2-34"],
    ["1 Chr. 8", "1 Cor. 12 - 13"],
    ["1 Chr. 10", "1 Cor. 14"],
    ["1 Chr. 12", "1 Cor. 15"],
    ["1 Chr. 14", "1 Cor. 16"],
    ["1 Chr. 16", "2 Cor. 1:1 - 2:4"],
    ["1 Chr. 18", "2 Cor. 2:5 - 3:18"],
    ["1 Chr. 21:1 - 22:1", "2 Cor. 4 - 5:10"],
    ["1 Chr. 23", "2 Cor. 5:11 - 7:1"],
    ["1 Chr. 25", "2 Cor. 7:2 - 8:24"],
    ["1 Chr. 27", "2 Cor. 9"],
    ["1 Chr. 29", "2 Cor. 10"],
    ["2 Chr. 2", "2 Cor. 11"],
    ["2 Chr. 4:1 - 5:1", "2 Cor. 12"],
    ["2 Chr. 6", "2 Cor. 13"],
    ["2 Chr. 8", "Eph. 1"],
    ["2 Chr. 10", "Eph. 2"],
    ["2 Chr. 12", "Eph. 3"],
    ["2 Chr. 14", "Eph. 4"],
    ["2 Chr. 16", "Eph. 5"],
    ["2 Chr. 18", "Eph. 6"],
    ["2 Chr. 20", "Phil. 1"],
    ["2 Chr. 22", "Phil. 2"],
    ["2 Chr. 24", "Phil. 3:1 - 4:1"],
    ["2 Chr. 26", "Phil. 4:2-23"],
    ["2 Chr. 29", "Luke 1"],
    ["2 Chr. 31", "Luke 2"],
    ["2 Chr. 33", "Luke 3"],
    ["2 Chr. 35", "Luke 4"],
    ["Ezra 1", "Luke 5"],
    ["Ezra 3", "Luke 6"],
    ["Ezra 5", "Luke 7"],
    ["Ezra 7", "Luke 8"],
    ["Ezra 9", "Luke 9"],
    ["Neh. 1", "Luke 10"],
    ["Neh. 3", "Luke 11"],
    ["Neh. 5", "Luke 12"],
    ["Neh. 7:5-73", "Luke 13"],
    ["Neh. 9", "Luke 14"],
    ["Neh. 11", "Luke 15"],
    ["Neh. 13", "Luke 16"],
    ["Est. 2", "Luke 17"],
    ["Est. 5 - 6", "Luke 18"],
    ["Est. 9 - 10", "Luke 19"],
    ["Job 2", "Luke 20"],
    ["Job 4", "Luke 21"],
    ["Job 6", "Luke 22"],
    ["Job 8", "Luke 23"],
    ["Job 10", "Luke 24"],
    ["Job 12", "2 Tim. 1"],
    ["Job 14", "2 Tim. 2"],
    ["Job 16", "2 Tim. 3"],
    ["Job 18", "2 Tim. 4"],
    ["Job 20", "Titus 1"],
    ["Job 22", "Titus 2 - 3"],
    ["Job 24", "Philemon"],
    ["Job 27", "Acts 1"],
    ["Job 29", "Acts 2"],
    ["Job 31", "Acts 3"],
    ["Job 33", "Acts 4"],
    ["Job 35", "Acts 5 - 6"],
    ["Job 37", "Acts 7"],
    ["Job 39", "Acts 8"],
    ["Job 41", "Acts 9"],
    ["Prov. 1", "Acts 10"],
    ["Prov. 3", "Acts 11 - 12"],
    ["Prov. 5", "Acts 13"],
    ["Prov. 7", "Acts 14"],
    ["Prov. 9", "Acts 15"],
    ["Prov. 11", "Acts 16"],
    ["Prov. 13", "Acts 17"],
    ["Prov. 15", "Acts 18"],
    ["Prov. 17", "Acts 19"],
    ["Prov. 19", "Acts 20"],
    ["Prov. 21", "Acts 21:1-36"],
    ["Prov. 23", "Acts 21:37 - 22:29"],
    ["Prov. 25", "Acts 22:30 - 23:35"],
    ["Prov. 27", "Acts 24"],
    ["Prov. 29", "Acts 25 - 26"],
    ["Prov. 31", "Acts 27"],
    ["Ecc. 3", "Acts 28"],
    ["Ecc. 6 - 7", "1 John 1:1 - 2:14"],
    ["Ecc. 9", "1 John 2:15 - 3:10"],
    ["Ecc. 11 - 12", "1 John 3:11-24"],
    ["So. 2", "1 John 4"],
    ["So. 4:1-16a", "1 John 5"],
    ["So. 6:2-13", "2 John, 3 John"],
    ["So. 8", "Jude"],
    ["Jer. 2", "Matt. 1"],
    ["Jer. 4", "Matt. 2"],
    ["Jer. 6", "Matt. 3"],
    ["Jer. 8:4-21", "Matt. 4"],
    ["Jer. 10", "Matt. 5"],
    ["Jer. 12", "Matt. 6"],
    ["Jer. 14", "Matt. 7"],
    ["Jer. 16", "Matt. 8"],
    ["Jer. 18", "Matt. 9"],
    ["Jer. 20", "Matt. 10"],
    ["Jer. 22", "Matt. 11"],
    ["Jer. 24", "Matt. 12"],
    ["Jer. 26", "Matt. 13"],
    ["Jer. 28", "Matt. 14"],
    ["Jer. 30", "Matt. 15"],
    ["Jer. 32", "Matt. 16"],
    ["Jer. 34", "Matt. 17"],
    ["Jer. 36", "Matt. 18"],
    ["Jer. 38", "Matt. 19"],
    ["Jer. 40", "Matt. 20"],
    ["Jer. 42", "Matt. 21"],
    ["Jer. 44", "Matt. 22"],
    ["Jer. 47", "Matt. 23"],
    ["Jer. 49", "Matt. 24"],
    ["Jer. 51", "Matt. 25"],
    ["Lam. 1", "Matt. 26"],
    ["Lam. 3", "Matt. 27"],
    ["Lam. 5", "Matt. 28"],
    ["Ezek. 2:1 - 3:15", "2 Cor. 1:1 - 2:4"],
    ["Ezek. 5", "2 Cor. 2:5 - 3:18"],
    ["Ezek. 7", "2 Cor. 4 - 5:10"],
    ["Ezek. 9", "2 Cor. 5:11 - 7:1"],
    ["Ezek. 11", "2 Cor. 7:2 - 8:24"],
    ["Ezek. 13", "2 Cor. 9"],
    ["Ezek. 16", "2 Cor. 10"],
    ["Ezek. 18", "2 Cor. 11"],
    ["Ezek. 20", "2 Cor. 12"],
    ["Ezek. 22", "2 Cor. 13"],
    ["Ezek. 24", "Rom. 1"],
    ["Ezek. 26", "Rom. 2"],
    ["Ezek. 28", "Rom. 3"],
    ["Ezek. 30", "Rom. 4"],
    ["Ezek. 32", "Rom. 5"],
    ["Ezek. 34 - 35", "Rom. 6"],
    ["Ezek. 37", "Rom. 7"],
    ["Ezek. 39", "Rom. 8"],
    ["Ezek. 41", "Rom. 9:1-29"],
    ["Ezek. 43", "Rom. 9:30 - 10:21"],
    ["Ezek. 45", "Rom. 11"],
    ["Ezek. 47", "Rom. 12"],
    ["Dan. 1", "Rom. 13"],
    ["Dan. 3", "Rom. 14"],
    ["Dan. 5", "Rom. 15"],
    ["Dan. 7", "Rom. 16"],
    ["Dan. 9", "1 Thess. 1:1 - 2:16"],
    ["Dan. 11", "1 Thess. 2:17 - 3:13"],
    ["Hos. 1:1 - 2:15", "1 Thess. 4"],
    ["Hos. 5", "1 Thess. 5"],
    ["Hos. 7", "2 Thess. 1"],
    ["Hos. 9", "2 Thess. 2"],
    ["Hos. 11", "2 Thess. 3"],
    ["Hos. 13", "Mark 1"],
    ["Joel 1", "Mark 2"],
    ["Joel 3", "Mark 3"],
    ["Amos 2", "Mark 4"],
    ["Amos 4", "Mark 5"],
    ["Amos 6", "Mark 6"],
    ["Amos 8", "Mark 7"],
    ["Obab.", "Mark 8:1 - 9:1"],
    ["Jonah 2 - 3", "Mark 9:2-50"],
    ["Mic. 1", "Mark 10"],
    ["Mic. 3", "Mark 11"],
    ["Mic. 5", "Mark 12"],
    ["Mic. 7", "Mark 13"],
    ["Nah. 2", "Mark 14"],
    ["Hab. 1:1 - 2:1", "Mark 15"],
    ["Hab. 3", "Mark 16"],
    ["Zeph. 2", "1 Tim. 1"],
    ["Hag. 1", "1 Tim. 2 - 3"],
    ["Zech. 1", "1 Tim. 4"],
    ["Zech. 4 - 5", "1 Tim. 5:1 - 6:2a"],
    ["Zech. 7", "1 Tim. 6:2b-21"],
    ["Zech. 9", "2 Tim. 1"],
    ["Zech. 11:2-17", "2 Tim. 2"],
    ["Zech. 13:2-9", "2 Tim. 3"],
    ["Mal. 1", "2 Tim. 4"],
    ["Mal. 2:17 - 3:15", "Titus 1"],
    ["Isa. 1", "Titus 2 - 3"],
    ["Isa. 3:1 - 4:1", "Philemon"],
    ["Isa. 5:8-30", "Acts 1"],
    ["Isa. 7", "Acts 2"],
    ["Isa. 9:1 - 10:4", "Acts 3"],
    ["Isa. 11", "Acts 4"],
    ["Isa. 14", "Acts 5"],
    ["Isa. 16", "Acts 6"],
    ["Isa. 19", "Acts 7"],
    ["Isa. 21", "Acts 8"],
    ["Isa. 23", "Acts 9"],
    ["Isa. 25", "Acts 10"],
    ["Isa. 27", "Acts 11 - 12"],
    ["Isa. 29", "Acts 13"],
    ["Isa. 31", "Acts 14"],
    ["Isa. 33", "Acts 15"],
    ["Isa. 35", "Acts 16"],
    ["Isa. 37", "Acts 17"],
    ["Isa. 39", "Acts 18"],
    ["Isa. 41", "Acts 19"],
    ["Isa. 43", "Acts 20"],
    ["Isa. 45", "Acts 21:1-36"],
    ["Isa. 47", "Acts 21:37 - 22:29"],
    ["Isa. 49", "Acts 22:30 - 23:35"],
    ["Isa. 51", "Acts 24"],
    ["Isa. 53", "Acts 25 - 26"],
    ["Isa. 9:1-17", "Luke 2:1-14"],
    ["Isa. 55", "Acts 27"],
    ["Isa. 57", "Acts 28"],
    ["Isa. 59", "1 John 1:1 - 2:14"],
    ["Isa. 61", "1 John 2:15 - 3:10"],
    ["Isa. 63", "1 John 5"],
    ["Isa. 65", "3 John"],
];

static EVENING: [[&'static str; 2]; 360] = [
    ["Gen. 2", "1 Pet. 1"],
    ["Gen. 4 - 5", "1 Pet. 2"],
    ["Gen. 8 - 9", "1 Pet. 3"],
    ["Gen. 12 - 13", "1 Pet. 4"],
    ["Gen. 16 - 17", "1 Pet. 5"],
    ["Gen. 19", "2 Pet. 1"],
    ["Gen. 22 - 23", "2 Pet. 2"],
    ["Gen. 25 - 26", "2 Pet. 3"],
    ["Gen. 28:1 - 29:30", "Gal. 1"],
    ["Gen. 31", "Gal. 2"],
    ["Gen. 34 - 35", "Gal. 3"],
    ["Gen. 37", "Gal. 4"],
    ["Gen. 39 - 40", "Gal. 5"],
    ["Gen. 42", "Gal. 6"],
    ["Gen. 44 - 45", "Eph. 1"],
    ["Gen. 47", "Eph. 2"],
    ["Gen. 49:1-27", "Eph. 3"],
    ["Ex. 1 - 2", "Eph. 4"],
    ["Ex. 5 - 6", "Eph. 5"],
    ["Ex. 9", "Eph. 6"],
    ["Ex. 12", "James 1"],
    ["Ex. 14", "James 2"],
    ["Ex. 16", "James 3"],
    ["Ex. 19 - 20", "James 4"],
    ["Ex. 22:16 - 23:33", "James 5"],
    ["Ex. 26 - 27", "Matt. 1"],
    ["Ex. 29", "Matt. 2"],
    ["Ex. 32", "Matt. 3"],
    ["Ex. 35 - 36", "Matt. 4"],
    ["Ex. 39", "Matt. 5"],
    ["Lev. 1 - 2", "Matt. 6"],
    ["Lev. 5", "Matt. 7"],
    ["Lev. 7", "Matt. 8"],
    ["Lev. 9", "Matt. 9"],
    ["Lev. 11 - 12", "Matt. 10"],
    ["Lev. 14", "Matt. 11"],
    ["Lev. 16", "Matt. 12"],
    ["Lev. 19", "Matt. 13"],
    ["Lev. 21", "Matt. 14"],
    ["Lev. 23", "Matt. 15"],
    ["Lev. 25", "Matt. 16"],
    ["Lev. 27", "Matt. 17"],
    ["Num. 2", "Matt. 18"],
    ["Num. 4", "Matt. 19"],
    ["Num. 6", "Matt. 20"],
    ["Num. 8", "Matt. 21"],
    ["Num. 10", "Matt. 22"],
    ["Num. 12 - 13", "Matt. 23"],
    ["Num. 15", "Matt. 24"],
    ["Num. 17 - 18", "Matt. 25"],
    ["Num. 20", "Matt. 26"],
    ["Num. 22", "Matt. 27"],
    ["Num. 24", "Matt. 28"],
    ["Num. 26", "Phil. 1"],
    ["Num. 28", "Phil. 2"],
    ["Num. 30", "Phil. 3:1 - 4:1"],
    ["Num. 32", "Phil. 4:2-23"],
    ["Num. 34:1 - 35:8", "Col. 1:1 - 2:5"],
    ["Joel 2", "2 Pet. 3"],
    ["Deut. 1", "Col. 2:6-23"],
    ["Deut. 3", "Col. 3:1-17"],
    ["Deut. 5", "Col. 3:18 - 4:18"],
    ["Deut. 7", "1 Thess. 1:1 - 2:16"],
    ["Deut. 9", "1 Thess. 2:17 - 3:13"],
    ["Deut. 11", "1 Thess. 4"],
    ["Deut. 13", "1 Thess. 5"],
    ["Deut. 16", "2 Thess. 1"],
    ["Deut. 18", "2 Thess. 2"],
    ["Deut. 20", "2 Thess. 3"],
    ["Deut. 22", "Rom. 1"],
    ["Deut. 24:1 - 25:4", "Rom. 2"],
    ["Deut. 27", "Rom. 3"],
    ["Deut. 29", "Rom. 4"],
    ["Deut. 31", "Rom. 5"],
    ["Deut. 33", "Rom. 6"],
    ["Josh. 1", "Rom. 7"],
    ["Josh. 3", "Rom. 8"],
    ["Josh. 5", "Rom. 9:1-29"],
    ["Josh. 7", "Rom. 9:30 - 10:21"],
    ["Josh. 9", "Rom. 11"],
    ["Josh. 11", "Rom. 12"],
    ["Josh. 13", "Rom. 13"],
    ["Josh. 15", "Rom. 14"],
    ["Josh. 17", "Rom. 15"],
    ["Josh. 19", "Rom. 16"],
    ["Josh. 22", "Titus 1"],
    ["Judg. 1", "Titus 2 - 3"],
    ["Judg. 3:7-31", "Philemon"],
    ["Judg. 5", "Luke 1"],
    ["Judg. 7", "Luke 2"],
    ["Judg. 9", "Luke 3"],
    ["Judg. 11", "Luke 4"],
    ["Judg. 13", "Luke 5"],
    ["Judg. 15", "Luke 6"],
    ["Judg. 17", "Luke 7"],
    ["Judg. 19", "Luke 8"],
    ["Judg. 21", "Luke 9"],
    ["Ruth 2", "Luke 10"],
    ["1 Sam. 2", "Luke 11"],
    ["1 Sam. 4", "Luke 12"],
    ["1 Sam. 6:1 - 7:2", "Luke 13"],
    ["1 Sam. 8", "Luke 14"],
    ["1 Sam. 10", "Luke 15"],
    ["1 Sam. 12", "Luke 16"],
    ["1 Sam. 14", "Luke 17"],
    ["1 Sam. 16", "Luke 18"],
    ["1 Sam. 18", "Luke 19"],
    ["1 Sam. 20", "Luke 20"],
    ["1 Sam. 22", "Luke 21"],
    ["1 Sam. 24", "Luke 22"],
    ["1 Sam. 26", "Luke 23"],
    ["1 Sam. 28", "Luke 24"],
    ["1 Sam. 30", "Rev. 1"],
    ["2 Sam. 1", "Rev. 2"],
    ["2 Sam. 3", "Rev. 3"],
    ["2 Sam. 5", "Rev. 4"],
    ["2 Sam. 7", "Rev. 5"],
    ["2 Sam. 9", "Rev. 6"],
    ["2 Sam. 11", "Rev. 7"],
    ["2 Sam. 13", "Rev. 8 - 9"],
    ["2 Sam. 15", "Rev. 10"],
    ["2 Sam. 17", "Rev. 11"],
    ["2 Sam. 19", "Rev. 12"],
    ["2 Sam. 21", "Rev. 13"],
    ["2 Sam. 23", "Rev. 14"],
    ["1 Kings 1", "Rev. 15"],
    ["1 Kings 3", "Rev. 16"],
    ["1 Kings 5", "Rev. 17"],
    ["1 Kings 7", "Rev. 18"],
    ["1 Kings 9", "Rev. 19"],
    ["1 Kings 11", "Rev. 20"],
    ["1 Kings 13", "Rev. 21"],
    ["1 Kings 15:1 - 16:7", "Rev. 22"],
    ["1 Kings 17", "Heb. 1"],
    ["1 Kings 19", "Heb. 2"],
    ["1 Kings 21", "Heb. 3"],
    ["2 Kings 1", "Heb. 4:1 - 13"],
    ["2 Kings 3", "Heb. 4:14 - 5:10"],
    ["2 Kings 5", "Heb. 5:11 - 6:20"],
    ["2 Kings 7", "Heb. 7"],
    ["2 Kings 9", "Heb. 8"],
    ["2 Kings 11", "Heb. 9"],
    ["2 Kings 13", "Heb. 10"],
    ["2 Kings 15", "Heb. 11"],
    ["2 Kings 16:1 - 17:5", "Heb. 12"],
    ["2 Kings 18", "Heb. 13"],
    ["2 Kings 20", "Mark 1"],
    ["2 Kings 22", "Mark 2"],
    ["2 Kings 23:36 - 24:20", "Mark 3"],
    ["1 Chr. 1", "Mark 4"],
    ["1 Chr. 3", "Mark 5"],
    ["1 Chr. 5", "Mark 6"],
    ["1 Chr. 7", "Mark 7"],
    ["1 Chr. 9", "Mark 8:1 - 9:1"],
    ["1 Chr. 11", "Mark 9:2-50"],
    ["1 Chr. 13", "Mark 10"],
    ["1 Chr. 15", "Mark 11"],
    ["1 Chr. 17", "Mark 12"],
    ["1 Chr. 19 - 20", "Mark 13"],
    ["1 Chr. 22:2-19", "Mark 14"],
    ["1 Chr. 24", "Mark 15"],
    ["1 Chr. 26", "Mark 16"],
    ["1 Chr. 28", "Rom. 1"],
    ["2 Chr. 1", "Rom. 2"],
    ["2 Chr. 3", "Rom. 3"],
    ["2 Chr. 5:2-14", "Rom. 4"],
    ["2 Chr. 7", "Rom. 5"],
    ["2 Chr. 9", "Rom. 6"],
    ["2 Chr. 11", "Rom. 7"],
    ["2 Chr. 13", "Rom. 8"],
    ["2 Chr. 15", "Rom. 9:1-29"],
    ["2 Chr. 17", "Rom. 9:30 - 10:21"],
    ["2 Chr. 19", "Rom. 11"],
    ["2 Chr. 21", "Rom. 12"],
    ["2 Chr. 23", "Rom. 13"],
    ["2 Chr. 25", "Rom. 14"],
    ["2 Chr. 27 - 28", "Rom. 15"],
    ["2 Chr. 30", "Rom. 16"],
    ["2 Chr. 32", "Gal. 1"],
    ["2 Chr. 34", "Gal. 2"],
    ["2 Chr. 36", "Gal. 3"],
    ["Ezra 2", "Gal. 4"],
    ["Ezra 4", "Gal. 5"],
    ["Ezra 6", "Gal. 6"],
    ["Ezra 8", "Col. 1:1 - 2:5"],
    ["Ezra 10", "Col. 2:6-23"],
    ["Neh. 2", "Col. 3:1-17"],
    ["Neh. 4", "Col. 3:18 - 4:18"],
    ["Neh. 6:1 - 7:4", "1 Thess. 1:1 - 2:16"],
    ["Neh. 8", "1 Thess. 2:17 - 3:13"],
    ["Neh. 10", "1 Thess. 4"],
    ["Neh. 12", "1 Thess. 5"],
    ["Est. 1", "2 Thess. 1"],
    ["Est. 3 - 4", "2 Thess. 2"],
    ["Est. 7 - 8", "2 Thess. 3"],
    ["Job 1", "1 Tim. 1"],
    ["Job 3", "1 Tim. 2 - 3"],
    ["Job 5", "1 Tim. 4"],
    ["Job 7", "1 Tim. 5:1 - 6:2a"],
    ["Job 9", "1 Tim. 6:2b-21"],
    ["Job 11", "James 1"],
    ["Job 13", "James 2"],
    ["Job 15", "James 3"],
    ["Job 17", "James 4"],
    ["Job 19", "James 5"],
    ["Job 21", "1 Pet. 1"],
    ["Job 23", "1 Pet. 2"],
    ["Job 25 - 26", "1 Pet. 3"],
    ["Job 28", "1 Pet. 4"],
    ["Job 30", "1 Pet. 5"],
    ["Job 32", "2 Pet. 1"],
    ["Job 34", "2 Pet. 2"],
    ["Job 36", "2 Pet. 3"],
    ["Job 38", "John 1"],
    ["Job 40", "John 2"],
    ["Job 42", "John 3"],
    ["Prov. 2", "John 4"],
    ["Prov. 4", "John 5"],
    ["Prov. 6", "John 6"],
    ["Prov. 8", "John 7"],
    ["Prov. 10", "John 8"],
    ["Prov. 12", "John 9"],
    ["Prov. 14", "John 10"],
    ["Prov. 16", "John 11"],
    ["Prov. 18", "John 12"],
    ["Prov. 20", "John 13"],
    ["Prov. 22", "John 14:1 - 16:4a"],
    ["Prov. 24", "John 16:4b-33"],
    ["Prov. 26", "John. 17"],
    ["Prov. 28", "John. 18"],
    ["Prov. 30", "John. 19"],
    ["Ecc. 1 - 2", "John. 20"],
    ["Ecc. 4 - 5", "John. 21"],
    ["Ecc. 8", "Rev. 1"],
    ["Ecc. 10", "Rev. 2"],
    ["So. 1", "Rev. 3"],
    ["So. 3", "Rev. 4"],
    ["So. 4:16b - 6:1", "Rev. 5"],
    ["So. 7", "Rev. 6"],
    ["Jer. 1", "Rev. 7"],
    ["Jer. 3", "Rev. 8 - 9"],
    ["Jer. 5", "Rev. 10"],
    ["Jer. 7:1 - 8:3", "Rev. 11"],
    ["Jer. 8:22 - 9:26", "Rev. 12"],
    ["Jer. 11", "Rev. 13"],
    ["Jer. 13", "Rev. 14"],
    ["Jer. 15", "Rev. 15"],
    ["Jer. 17", "Rev. 16"],
    ["Jer. 19", "Rev. 17"],
    ["Jer. 21", "Rev. 18"],
    ["Jer. 23", "Rev. 19"],
    ["Jer. 25", "Rev. 20"],
    ["Jer. 27", "Rev. 21"],
    ["Jer. 29", "Rev. 22"],
    ["Jer. 31", "1 Cor. 1"],
    ["Jer. 33", "1 Cor. 2"],
    ["Jer. 35", "1 Cor. 3"],
    ["Jer. 37", "1 Cor. 4"],
    ["Jer. 39", "1 Cor. 5 - 6"],
    ["Jer. 41", "1 Cor. 7"],
    ["Jer. 43", "1 Cor. 8"],
    ["Jer. 45 - 46", "1 Cor. 9"],
    ["Jer. 48", "1 Cor. 10:1 - 11:1"],
    ["Jer. 50", "1 Cor. 11:2-34"],
    ["Jer. 52", "1 Cor. 12 - 13"],
    ["Lam. 2", "1 Cor. 14"],
    ["Lam. 4", "1 Cor. 15"],
    ["Ezek. 1", "1 Cor. 16"],
    ["Ezek. 3:16 - 4:17", "John 1"],
    ["Ezek. 6", "John 2"],
    ["Ezek. 8", "John 3"],
    ["Ezek. 10", "John 4"],
    ["Ezek. 12", "John 5"],
    ["Ezek. 14 - 15", "John 6"],
    ["Ezek. 17", "John 7"],
    ["Ezek. 19", "John 8"],
    ["Ezek. 21", "John 9"],
    ["Ezek. 23", "John 10"],
    ["Ezek. 25", "John 11"],
    ["Ezek. 27", "John 12"],
    ["Ezek. 29", "John 13"],
    ["Ezek. 31", "John 14:1 - 16:4a"],
    ["Ezek. 33", "John 16:4b-33"],
    ["Ezek. 36", "John. 17"],
    ["Ezek. 38", "John. 18"],
    ["Ezek. 40", "John. 19"],
    ["Ezek. 42", "John. 20"],
    ["Ezek. 44", "John. 21"],
    ["Ezek. 46", "Gal. 1"],
    ["Ezek. 48", "Gal. 2"],
    ["Dan. 2", "Gal. 3"],
    ["Dan. 4", "Gal. 4"],
    ["Dan. 6", "Gal. 5"],
    ["Dan. 8", "Gal. 6"],
    ["Dan. 10", "Eph. 1"],
    ["Dan. 12", "Eph. 2"],
    ["Hos. 2:16 - 4:19", "Eph. 3"],
    ["Hos. 6", "Eph. 4"],
    ["Hos. 8", "Eph. 5"],
    ["Hos. 10", "Eph. 6"],
    ["Hos. 12", "Phil. 1"],
    ["Hos. 14", "Phil. 2"],
    ["Joel 2", "Phil. 3:1 - 4:1"],
    ["Amos 1", "Phil. 4:2-23"],
    ["Amos 3", "Col. 1:1 - 2:5"],
    ["Amos 5", "Col. 2:6-23"],
    ["Amos 7", "Col. 3:1-17"],
    ["Amos 9", "Col. 3:18 - 4:18"],
    ["Jonah 1", "Heb. 1"],
    ["Jonah 4", "Heb. 2"],
    ["Mic. 2", "Heb. 3"],
    ["Mic. 4", "Heb. 4:1 - 13"],
    ["Mic. 6", "Heb. 4:14 - 5:10"],
    ["Nah. 1", "Heb. 5:11 - 6:20"],
    ["Nah. 3", "Heb. 7"],
    ["Hab. 2:1-20", "Heb. 8"],
    ["Zeph. 1", "Heb. 9"],
    ["Zeph. 3", "Heb. 10"],
    ["Hag. 2", "Heb. 11"],
    ["Zech. 2 - 3", "Heb. 12 - 13"],
    ["Zech. 6", "James 1"],
    ["Zech. 8", "James 2"],
    ["Zech. 10:1 - 11:1", "James 3 - 4"],
    ["Zech. 12:1 - 13:1", "James 5"],
    ["Zech. 14", "1 Pet. 1"],
    ["Mal. 2:1-16", "1 Pet. 2"],
    ["Mal. 3:16 - 4:6", "1 Pet. 3"],
    ["Isa. 2", "1 Pet. 4 - 5"],
    ["Isa. 4:2 - 5:7", "2 Pet. 1"],
    ["Isa. 6", "2 Pet. 2"],
    ["Isa. 8", "2 Pet. 3"],
    ["Isa. 10:5-34", "Luke 1"],
    ["Isa. 12 - 13", "Luke 2"],
    ["Isa. 15", "Luke 3"],
    ["Isa. 17", "Luke 4"],
    ["Isa. 20", "Luke 5"],
    ["Isa. 22", "Luke 6"],
    ["Isa. 24", "Luke 7"],
    ["Isa. 26", "Luke 8"],
    ["Isa. 28", "Luke 9"],
    ["Isa. 30", "Luke 10"],
    ["Isa. 32", "Luke 11"],
    ["Isa. 34", "Luke 12"],
    ["Isa. 36", "Luke 13"],
    ["Isa. 38", "Luke 14"],
    ["Isa. 40", "Luke 15"],
    ["Isa. 42", "Luke 16"],
    ["Isa. 44", "Luke 17"],
    ["Isa. 46", "Luke 18"],
    ["Isa. 48", "Luke 19"],
    ["Isa. 50", "Luke 20"],
    ["Isa. 52", "Luke 21"],
    ["Isa. 54", "Luke 22"],
    ["Isa. 7:10-16", "Titus 3:4-8"],
    ["Isa. 56", "Luke 23"],
    ["Isa. 58", "Luke 24"],
    ["Isa. 60", "1 John 3:11-24"],
    ["Isa. 62", "1 John 4"],
    ["Isa. 64", "2 John"],
    ["Isa. 66", "Jude"],
];
