//! Layout placement for native overview cards in single and two-column layouts.
//!
//! What: places status, folders, settings and info cards in calculated grid slots.
//! Why: maintain modularity and stay under function/file budget.

use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::grid::{
    GridLayout, BUTTON_HEIGHT, FIELD_HEIGHT, GAP, LABEL_HEIGHT, PAD, PICKER_HEIGHT, TIGHT_GAP,
};
use super::reflow::{place, Heights};
use super::OverviewViews;

/// Places the status card.
pub(super) fn place_status_card(
    views: &OverviewViews,
    heights: &Heights,
    grid: &GridLayout,
    top: f64,
    height: f64,
) -> f64 {
    views.status_card.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x, top - height),
        NSSize::new(grid.col_width, height),
    ));
    let mut inner = top - PAD;
    let toggle_w = (grid.inner_width * 0.4).min(130.0);
    place(
        &views.status,
        grid.col1_x + PAD,
        grid.inner_width - toggle_w - 10.0,
        26.0,
        &mut inner,
        TIGHT_GAP,
    );
    views.controls.toggle.setFrame(NSRect::new(
        NSPoint::new(
            grid.col1_x + grid.col_width - PAD - toggle_w,
            top - PAD - BUTTON_HEIGHT,
        ),
        NSSize::new(toggle_w, BUTTON_HEIGHT),
    ));
    place(
        &views.library,
        grid.col1_x + PAD,
        grid.inner_width,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    if heights.notice > 0.0 {
        place(
            &views.notice,
            grid.col1_x + PAD,
            grid.inner_width,
            heights.notice,
            &mut inner,
            0.0,
        );
    }
    top - height - GAP
}

/// Places the folders card and action buttons.
pub(super) fn place_folders_card(
    views: &OverviewViews,
    heights: &Heights,
    grid: &GridLayout,
    top: f64,
    height: f64,
) -> f64 {
    views.folders_card.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x, top - height),
        NSSize::new(grid.col_width, height),
    ));
    let mut inner = top - PAD;
    place(
        &views.folders_heading,
        grid.col1_x + PAD,
        grid.inner_width,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.folders,
        grid.col1_x + PAD,
        grid.inner_width,
        heights.folders,
        &mut inner,
        GAP,
    );
    views.controls.folder_picker.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x + PAD, inner - PICKER_HEIGHT),
        NSSize::new(grid.inner_width, PICKER_HEIGHT),
    ));
    inner -= PICKER_HEIGHT + TIGHT_GAP;

    // Distribute buttons across two rows or grid:
    // Row 1: Finder-Anzeigen, Ordner entfernen
    // Row 2: Ordner hinzufügen, Neu einlesen
    let btn_gap = 8.0;
    let btn_w = ((grid.inner_width - btn_gap) / 2.0).floor().max(80.0);

    // Row 1: Actions on selected folder
    views.controls.reveal_folder.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x + PAD, inner - BUTTON_HEIGHT),
        NSSize::new(btn_w, BUTTON_HEIGHT),
    ));
    views.controls.remove_folder.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x + PAD + btn_w + btn_gap, inner - BUTTON_HEIGHT),
        NSSize::new(btn_w, BUTTON_HEIGHT),
    ));
    inner -= BUTTON_HEIGHT + TIGHT_GAP;

    // Row 2: Global folder actions
    views.add_folders.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x + PAD, inner - BUTTON_HEIGHT),
        NSSize::new(btn_w, BUTTON_HEIGHT),
    ));
    views.rescan_button.setFrame(NSRect::new(
        NSPoint::new(grid.col1_x + PAD + btn_w + btn_gap, inner - BUTTON_HEIGHT),
        NSSize::new(btn_w, BUTTON_HEIGHT),
    ));

    top - height - GAP
}

/// Places the settings card.
pub(super) fn place_settings_card(
    views: &OverviewViews,
    grid: &GridLayout,
    top: f64,
    height: f64,
) -> f64 {
    let card_x = if grid.is_two_col {
        grid.col2_x
    } else {
        grid.col1_x
    };
    views.settings_card.setFrame(NSRect::new(
        NSPoint::new(card_x, top - height),
        NSSize::new(grid.col_width, height),
    ));
    let mut inner = top - PAD;
    let label_w = 110.0;
    let field_x = card_x + PAD + label_w + 10.0;
    let field_w = (grid.inner_width - label_w - 10.0).max(120.0);

    views.controls.device_name_label.setFrame(NSRect::new(
        NSPoint::new(card_x + PAD, inner - LABEL_HEIGHT - 3.0),
        NSSize::new(label_w, LABEL_HEIGHT),
    ));
    views.controls.device_name_field.setFrame(NSRect::new(
        NSPoint::new(field_x, inner - FIELD_HEIGHT),
        NSSize::new(field_w.min(260.0), FIELD_HEIGHT),
    ));
    inner -= FIELD_HEIGHT + TIGHT_GAP;

    views.controls.http_port_label.setFrame(NSRect::new(
        NSPoint::new(card_x + PAD, inner - LABEL_HEIGHT - 3.0),
        NSSize::new(label_w, LABEL_HEIGHT),
    ));
    views.controls.http_port_field.setFrame(NSRect::new(
        NSPoint::new(field_x, inner - FIELD_HEIGHT),
        NSSize::new(90.0, FIELD_HEIGHT),
    ));
    inner -= FIELD_HEIGHT + GAP;

    views.controls.save_settings.setFrame(NSRect::new(
        NSPoint::new(field_x, inner - BUTTON_HEIGHT),
        NSSize::new(field_w.min(210.0), BUTTON_HEIGHT),
    ));
    top - height - GAP
}

/// Places the info card.
pub(super) fn place_info_card(
    views: &OverviewViews,
    heights: &Heights,
    grid: &GridLayout,
    top: f64,
    height: f64,
) {
    let card_x = if grid.is_two_col {
        grid.col2_x
    } else {
        grid.col1_x
    };
    views.info_card.setFrame(NSRect::new(
        NSPoint::new(card_x, top - height),
        NSSize::new(grid.col_width, height),
    ));
    let mut inner = top - PAD;
    place(
        &views.connection_heading,
        card_x + PAD,
        grid.inner_width,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.connection,
        card_x + PAD,
        grid.inner_width,
        heights.connection,
        &mut inner,
        GAP,
    );
    place(
        &views.activity_heading,
        card_x + PAD,
        grid.inner_width,
        20.0,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.activity,
        card_x + PAD,
        grid.inner_width,
        heights.activity,
        &mut inner,
        TIGHT_GAP,
    );
    place(
        &views.read_only,
        card_x + PAD,
        grid.inner_width,
        heights.read_only,
        &mut inner,
        0.0,
    );
}
