fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icon.ico");
        res.set("ProductName", "TabletDriver");
        res.set("FileDescription", "Tablet Driver");
        res.set("CompanyName", "iSweat");
        res.compile().unwrap();
    }
}
