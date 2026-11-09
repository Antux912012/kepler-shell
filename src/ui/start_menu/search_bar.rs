use gtk4::prelude::*;
use gtk4::{Box, Entry, Label, ListBox, Orientation};

pub struct StartSearchBar {
    container: Box,
    entry: Entry,
    results_list: ListBox,
    calc_result_label: Label,
}

impl StartSearchBar {
    pub fn new(placeholder: &str) -> Self {
        let container = Box::new(Orientation::Vertical, 6);

        let entry = Entry::builder()
            .placeholder_text(placeholder)
            .primary_icon_name("system-search-symbolic")
            .css_classes(["start-search-bar"])
            .hexpand(true)
            .build();

        let calc_result_label = Label::builder()
            .halign(gtk4::Align::Start)
            .visible(false)
            .css_classes(["heading", "accent"])
            .margin_start(12)
            .build();

        let results_list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .visible(false)
            .build();

        let cr_clone = calc_result_label.clone();
        let list_clone = results_list.clone();

        entry.connect_changed(move |e| {
            let query = e.text().trim().to_string();
            if query.is_empty() {
                cr_clone.set_visible(false);
                list_clone.set_visible(false);
                return;
            }

            // Built-in Calculator evaluation
            if let Some(calc_val) = Self::evaluate_math(&query) {
                cr_clone.set_text(&format!("= {}", calc_val));
                cr_clone.set_visible(true);
            } else {
                cr_clone.set_visible(false);
            }
        });

        container.append(&entry);
        container.append(&calc_result_label);
        container.append(&results_list);

        Self {
            container,
            entry,
            results_list,
            calc_result_label,
        }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn focus(&self) {
        self.entry.grab_focus();
    }

    pub fn clear(&self) {
        self.entry.set_text("");
        self.calc_result_label.set_visible(false);
        self.results_list.set_visible(false);
    }

    fn evaluate_math(expr: &str) -> Option<f64> {
        let clean: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
        if clean.is_empty() {
            return None;
        }

        // Support simple operations: +, -, *, /
        let ops = ['+', '-', '*', '/'];
        for op in ops {
            if let Some(idx) = clean.rfind(op) {
                if idx > 0 && idx < clean.len() - 1 {
                    let left = clean[..idx].parse::<f64>().ok()?;
                    let right = clean[idx + 1..].parse::<f64>().ok()?;
                    return match op {
                        '+' => Some(left + right),
                        '-' => Some(left - right),
                        '*' => Some(left * right),
                        '/' => if right != 0.0 { Some(left / right) } else { None },
                        _ => None,
                    };
                }
            }
        }
        None
    }
}
