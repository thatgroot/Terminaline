use crate::app::{App, TAB_COUNT};
use crate::types::*;
use crate::utils::*;
use crossterm::event::KeyCode;

/// Handle a key press. Returns true if the app should quit.
pub fn handle_key(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('q') => return true,
        KeyCode::Esc => {
            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                let parent = std::path::Path::new(&app.disk_path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !parent.is_empty()
                    && parent.len()
                        >= app
                            .disk_list
                            .get(app.disk_cursor)
                            .map(|d| d.mount_point.len())
                            .unwrap_or(1)
                {
                    app.disk_path = parent;
                    app.disk_files =
                        scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                    app.disk_file_cursor = 0;
                } else {
                    app.disk_mode = DiskMode::Partitions;
                }
            } else {
                return true;
            }
        }
        // Tab switching: 1-9, 0 for tab 10
        KeyCode::Char('1') => app.tab = 0,
        KeyCode::Char('2') => app.tab = 1,
        KeyCode::Char('3') => app.tab = 2,
        KeyCode::Char('4') => app.tab = 3,
        KeyCode::Char('5') => app.tab = 4,
        KeyCode::Char('6') => app.tab = 5,
        KeyCode::Char('7') => app.tab = 6,
        KeyCode::Char('8') => app.tab = 7,
        KeyCode::Char('9') => app.tab = 8,
        KeyCode::Char('0') => app.tab = 9,
        // New tab shortcuts
        KeyCode::Char('p') | KeyCode::Char('P') => app.tab = 10,
        KeyCode::Char('b') | KeyCode::Char('B') => app.tab = 11,
        KeyCode::Char('u') | KeyCode::Char('U') => app.tab = 12,
        KeyCode::Char('a') | KeyCode::Char('A') => app.tab = 13,
        KeyCode::Char('x') | KeyCode::Char('X') => app.tab = 14,
        KeyCode::Char('l') | KeyCode::Char('L') => app.tab = 15,
        KeyCode::Char('w') | KeyCode::Char('W') => app.tab = 16,
        KeyCode::Char('t') | KeyCode::Char('T') => app.tab = 17,
        KeyCode::Tab => app.tab = (app.tab + 1) % TAB_COUNT,
        KeyCode::BackTab => {
            app.tab = if app.tab == 0 {
                TAB_COUNT - 1
            } else {
                app.tab - 1
            }
        }
        KeyCode::Enter => handle_enter(app),
        KeyCode::Char('s') => {
            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                app.disk_sort = match app.disk_sort {
                    SortMode::SizeDsc => SortMode::SizeAsc,
                    SortMode::SizeAsc => SortMode::NameAsc,
                    SortMode::NameAsc => SortMode::NameDsc,
                    SortMode::NameDsc => SortMode::SizeDsc,
                };
                app.disk_files =
                    scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                app.disk_file_cursor = 0;
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                app.disk_filter_system = !app.disk_filter_system;
                app.disk_files =
                    scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                app.disk_file_cursor = 0;
            }
        }
        KeyCode::Right => {
            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                if let Some(fe) = app.disk_files.get(app.disk_file_cursor) {
                    if fe.is_dir {
                        let new_path = fe.path.clone();
                        app.disk_path = new_path;
                        app.disk_files =
                            scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                        app.disk_file_cursor = 0;
                    }
                }
            } else {
                app.tab = (app.tab + 1) % TAB_COUNT;
            }
        }
        KeyCode::Left => {
            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                let parent = std::path::Path::new(&app.disk_path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let min_len = app
                    .disk_list
                    .get(app.disk_cursor)
                    .map(|d| d.mount_point.len())
                    .unwrap_or(1);
                if !parent.is_empty() && parent.len() >= min_len {
                    app.disk_path = parent;
                    app.disk_files =
                        scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                    app.disk_file_cursor = 0;
                } else {
                    app.disk_mode = DiskMode::Partitions;
                }
            } else {
                app.tab = if app.tab == 0 {
                    TAB_COUNT - 1
                } else {
                    app.tab - 1
                };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => handle_scroll_down(app),
        KeyCode::Up | KeyCode::Char('k') => handle_scroll_up(app),
        KeyCode::PageDown => handle_page_down(app),
        KeyCode::PageUp => handle_page_up(app),
        _ => {}
    }
    false
}

fn handle_enter(app: &mut App) {
    if app.tab == 4 {
        if app.disk_mode == DiskMode::Partitions {
            if let Some(d) = app.disk_list.get(app.disk_cursor) {
                app.disk_path = d.mount_point.clone();
                app.disk_files =
                    scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                app.disk_file_cursor = 0;
                app.disk_mode = DiskMode::Files;
            }
        } else {
            if let Some(fe) = app.disk_files.get(app.disk_file_cursor) {
                if fe.is_dir {
                    let new_path = fe.path.clone();
                    app.disk_path = new_path;
                    app.disk_files =
                        scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                    app.disk_file_cursor = 0;
                }
            }
        }
    }
}

fn handle_scroll_down(app: &mut App) {
    match app.tab {
        0 => app.ram_scroll = app.ram_scroll.saturating_add(1),
        1 | 2 => {
            app.region_scroll = app
                .region_scroll
                .saturating_add(1)
                .min(app.regions.len().saturating_sub(1))
        }
        3 => app.cpu_scroll = app.cpu_scroll.saturating_add(1),
        4 => {
            if app.disk_mode == DiskMode::Partitions {
                app.disk_cursor = app
                    .disk_cursor
                    .saturating_add(1)
                    .min(app.disk_list.len().saturating_sub(1));
            } else {
                app.disk_file_cursor = app
                    .disk_file_cursor
                    .saturating_add(1)
                    .min(app.disk_files.len().saturating_sub(1));
            }
        }
        5 => app.net_scroll = app.net_scroll.saturating_add(1),
        9 => {
            app.activity_scroll = app
                .activity_scroll
                .saturating_add(1)
                .min(app.activity_connections.len().saturating_sub(1))
        }
        10 => {
            app.process_scroll = app
                .process_scroll
                .saturating_add(1)
                .min(app.processes.len().saturating_sub(1))
        }
        11 => app.bt_scroll = app.bt_scroll.saturating_add(1),
        12 => app.usb_scroll = app.usb_scroll.saturating_add(1),
        13 => app.audio_scroll = app.audio_scroll.saturating_add(1),
        15 => {
            app.service_scroll = app
                .service_scroll
                .saturating_add(1)
                .min(app.services.len().saturating_sub(1))
        }
        17 => app.thermal_scroll = app.thermal_scroll.saturating_add(1),
        _ => {}
    }
}

fn handle_scroll_up(app: &mut App) {
    match app.tab {
        0 => app.ram_scroll = app.ram_scroll.saturating_sub(1),
        1 | 2 => app.region_scroll = app.region_scroll.saturating_sub(1),
        3 => app.cpu_scroll = app.cpu_scroll.saturating_sub(1),
        4 => {
            if app.disk_mode == DiskMode::Partitions {
                app.disk_cursor = app.disk_cursor.saturating_sub(1);
            } else {
                app.disk_file_cursor = app.disk_file_cursor.saturating_sub(1);
            }
        }
        5 => app.net_scroll = app.net_scroll.saturating_sub(1),
        9 => app.activity_scroll = app.activity_scroll.saturating_sub(1),
        10 => app.process_scroll = app.process_scroll.saturating_sub(1),
        11 => app.bt_scroll = app.bt_scroll.saturating_sub(1),
        12 => app.usb_scroll = app.usb_scroll.saturating_sub(1),
        13 => app.audio_scroll = app.audio_scroll.saturating_sub(1),
        15 => app.service_scroll = app.service_scroll.saturating_sub(1),
        17 => app.thermal_scroll = app.thermal_scroll.saturating_sub(1),
        _ => {}
    }
}

fn handle_page_down(app: &mut App) {
    match app.tab {
        0 => app.ram_scroll = app.ram_scroll.saturating_add(10),
        1 | 2 => {
            app.region_scroll = app
                .region_scroll
                .saturating_add(20)
                .min(app.regions.len().saturating_sub(1))
        }
        3 => app.cpu_scroll = app.cpu_scroll.saturating_add(10),
        4 => {
            if app.disk_mode == DiskMode::Partitions {
                app.disk_cursor = app
                    .disk_cursor
                    .saturating_add(10)
                    .min(app.disk_list.len().saturating_sub(1));
            } else {
                app.disk_file_cursor = app
                    .disk_file_cursor
                    .saturating_add(20)
                    .min(app.disk_files.len().saturating_sub(1));
            }
        }
        5 => app.net_scroll = app.net_scroll.saturating_add(10),
        9 => {
            app.activity_scroll = app
                .activity_scroll
                .saturating_add(20)
                .min(app.activity_connections.len().saturating_sub(1))
        }
        10 => {
            app.process_scroll = app
                .process_scroll
                .saturating_add(20)
                .min(app.processes.len().saturating_sub(1))
        }
        15 => {
            app.service_scroll = app
                .service_scroll
                .saturating_add(20)
                .min(app.services.len().saturating_sub(1))
        }
        _ => {}
    }
}

fn handle_page_up(app: &mut App) {
    match app.tab {
        0 => app.ram_scroll = app.ram_scroll.saturating_sub(10),
        1 | 2 => app.region_scroll = app.region_scroll.saturating_sub(20),
        3 => app.cpu_scroll = app.cpu_scroll.saturating_sub(10),
        4 => {
            if app.disk_mode == DiskMode::Partitions {
                app.disk_cursor = app.disk_cursor.saturating_sub(10);
            } else {
                app.disk_file_cursor = app.disk_file_cursor.saturating_sub(20);
            }
        }
        5 => app.net_scroll = app.net_scroll.saturating_sub(10),
        9 => app.activity_scroll = app.activity_scroll.saturating_sub(20),
        10 => app.process_scroll = app.process_scroll.saturating_sub(20),
        15 => app.service_scroll = app.service_scroll.saturating_sub(20),
        _ => {}
    }
}
