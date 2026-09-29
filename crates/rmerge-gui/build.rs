fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/icons/rmerge.ico");
        res.set("ProductName", "Git-Client");
        res.set("FileDescription", "Git-Client: Native 3-Way Merge & Git Client");
        res.set("LegalCopyright", "Copyright (C) 2026 AnibalGH");
        if let Err(e) = res.compile() {
            eprintln!("Advertencia: No se pudo compilar el recurso de icono de Windows: {e}");
        }
    }
}
