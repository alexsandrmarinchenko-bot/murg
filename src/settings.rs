use gtk::prelude::*;
use relm4::gtk;

pub fn build_window() -> gtk::Window {
    let window = gtk::Window::new();
    window.set_title(Some("Murglar Controller — Настройки"));
    window.set_default_size(760, 500);

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    layout.set_margin_top(20);
    layout.set_margin_bottom(20);
    layout.set_margin_start(20);
    layout.set_margin_end(20);
    layout.set_spacing(20);

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 8);
    sidebar.set_width_request(190);
    sidebar.add_css_class("settings-sidebar");

    let heading = gtk::Label::new(Some("НАСТРОЙКИ"));
    heading.set_halign(gtk::Align::Start);
    heading.add_css_class("eyebrow");
    sidebar.append(&heading);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 14);
    content.set_hexpand(true);
    content.set_margin_start(12);
    content.add_css_class("settings-content");

    let section_title = gtk::Label::new(Some("Внешний вид"));
    section_title.set_halign(gtk::Align::Start);
    section_title.add_css_class("track-title");

    let description = gtk::Label::new(Some("Настройки оформления Murglar Controller."));
    description.set_halign(gtk::Align::Start);
    description.set_wrap(true);
    description.add_css_class("artist");

    content.append(&section_title);
    content.append(&description);

    let theme_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    let theme_label = gtk::Label::new(Some("Цветовая тема"));
    theme_label.set_halign(gtk::Align::Start);
    theme_label.add_css_class("artist");
    theme_box.set_hexpand(true);
    theme_label.set_text("Choose a theme");

    let theme_grid = gtk::Box::new(gtk::Orientation::Vertical, 10);
    theme_grid.set_hexpand(true);

    for (label, id) in [
        ("Glass", "glass"),
        ("Matte Graphite", "matte"),
        ("Cyberpunk", "cyberpunk"),
        ("Midnight", "midnight"),
        ("Forest", "forest"),
        ("Amber", "amber"),
    ].iter() {
        let button = gtk::Button::with_label(label);
        button.set_size_request(200, 40);
        button.set_hexpand(true);
        button.set_halign(gtk::Align::Fill);
        button.add_css_class("theme-choice-button");
        let theme_id = (*id).to_owned();
        button.connect_clicked(move |_| crate::ui::apply_theme(&theme_id));
        theme_grid.append(&button);
    }

    let theme_scroll = gtk::ScrolledWindow::new();
    theme_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    theme_scroll.set_min_content_height(240);
    theme_scroll.set_vexpand(true);
    theme_scroll.set_child(Some(&theme_grid));
    theme_box.append(&theme_label);
    theme_box.append(&theme_scroll);
    theme_box.set_visible(false);
    content.append(&theme_box);
    let sections = [
        ("Внешний вид", "Настройка внешнего вида приложения."),
        ("Темы", "Choose a theme."),
        ("Стекло", "Прозрачность, размытие и оттенок фона."),
        ("Плеер", "Настройки отображения музыкального плеера."),
        ("Дополнительно", "Дополнительные параметры приложения."),
    ];

    for (name, details) in sections {
        let button = gtk::Button::with_label(name);
        button.set_halign(gtk::Align::Fill);
        button.add_css_class("settings-nav-button");

        let title = section_title.clone();
        let text = description.clone();
        let themes = theme_box.clone();
        let name = name.to_owned();
        let details = details.to_owned();

        button.connect_clicked(move |_| {
            title.set_text(&name);
            text.set_text(&details);
            themes.set_visible(name == "Темы");
        });
        sidebar.append(&button);
    }

    layout.append(&sidebar);
    layout.append(&content);
    window.set_child(Some(&layout));
    window
}
